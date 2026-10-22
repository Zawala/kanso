//! One dictation: a thread that owns the microphone, and a thread that owns
//! the model.
//!
//! The microphone thread resamples to 16 kHz as sound arrives and, every
//! 20 seconds or so, cuts the recording at a pause ([`chunk::plan_cut`]) and
//! hands the piece to the transcriber, so most of the work is done by the
//! time the person stops. The transcriber loads the model while the first
//! piece records. Between pieces it transcribes what was recorded since the
//! last cut, so the words show while they are said ([`Dictation::live`]).

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use crate::VoiceError;
use crate::audio::{MODEL_RATE, Resampler16k, for_each_mono, meter_level, rms};
use crate::chunk;
use crate::engine::Parakeet;
use crate::live::{LiveText, preview_due};

/// The longest dictation: 30 minutes.
pub const MAX_RECORDING: Duration = Duration::from_secs(30 * 60);

/// How often the microphone thread collects what the device recorded.
const POLL: Duration = Duration::from_millis(20);

/// How long the transcriber waits for a piece before it looks at the tail.
const PREVIEW_WAIT: Duration = Duration::from_millis(50);

/// The stretch of sound the meter shows: 50 ms at 16 kHz.
const LEVEL_WINDOW: usize = MODEL_RATE as usize / 20;

/// State both threads and the app's handle share.
struct Shared {
    /// Set by `stop`, `cancel`, or the transcriber when the model fails.
    stop: AtomicBool,
    /// Set by `cancel` or dropping the handle: discard everything.
    cancel: AtomicBool,
    recording: AtomicBool,
    /// The meter level, as `f32` bits.
    level: AtomicU32,
    /// 16 kHz samples recorded so far.
    samples: AtomicU64,
    /// What was recorded since the last piece was cut off, for the
    /// transcriber to look at while it waits; `None` once taken. A piece is
    /// sent with this held, so the tail taken after it never repeats it.
    tail: Mutex<Option<Vec<f32>>>,
    /// What has been said so far.
    live: Mutex<LiveText>,
}

/// What the microphone thread hands the transcriber.
enum Piece {
    /// 16 kHz mono audio, in order.
    Audio(Vec<f32>),
    /// Recording is over; `failure` says why if the device gave out.
    End { failure: Option<String> },
}

/// One dictation: records from the default input device and transcribes as
/// it goes.
pub struct Dictation {
    shared: Arc<Shared>,
    result: async_channel::Receiver<Result<String, VoiceError>>,
    /// Whether `stop` handed the result on, so dropping must not cancel.
    stopped: bool,
}

impl Dictation {
    /// Starts recording now; the model at `model_dir` (already downloaded
    /// and verified by the app) loads on the transcriber thread meanwhile.
    ///
    /// Fails at once without a microphone, when the person denied it, or
    /// when the device won't open. A model that won't load ends the
    /// recording and comes back from [`stop`](Self::stop) as
    /// [`VoiceError::Model`].
    // no-prop-test: needs a microphone; its parts have the tests.
    pub fn start(model_dir: &Path) -> Result<Dictation, VoiceError> {
        if crate::permission::status() == crate::permission::Permission::Denied {
            return Err(VoiceError::MicrophoneDenied);
        }
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            recording: AtomicBool::new(false),
            level: AtomicU32::new(0f32.to_bits()),
            samples: AtomicU64::new(0),
            tail: Mutex::new(None),
            live: Mutex::new(LiveText::default()),
        });
        let (pieces_tx, pieces_rx) = mpsc::channel();
        let (result_tx, result_rx) = async_channel::bounded(1);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);

        let dir = model_dir.to_path_buf();
        let for_transcriber = shared.clone();
        thread::Builder::new()
            .name("kanso-voice-transcriber".into())
            .spawn(move || {
                let outcome = transcribe_pieces(&dir, &pieces_rx, &for_transcriber);
                let _ = result_tx.force_send(outcome);
            })
            .map_err(|e| VoiceError::Device(format!("can't start the transcriber: {e}")))?;

        let for_microphone = shared.clone();
        let started = thread::Builder::new()
            .name("kanso-voice-microphone".into())
            .spawn(move || record(&for_microphone, &pieces_tx, &ready_tx));
        let ready = match started {
            Ok(_) => ready_rx.recv().unwrap_or_else(|_| {
                Err(VoiceError::Device("the microphone thread stopped".into()))
            }),
            Err(e) => Err(VoiceError::Device(format!(
                "can't start the microphone thread: {e}"
            ))),
        };
        match ready {
            Ok(()) => Ok(Dictation {
                shared,
                result: result_rx,
                stopped: false,
            }),
            Err(e) => {
                shared.cancel.store(true, Ordering::Relaxed);
                shared.stop.store(true, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Recent loudness for a meter, 0 to 1 (see [`meter_level`]).
    pub fn level(&self) -> f32 {
        f32::from_bits(self.shared.level.load(Ordering::Relaxed))
    }

    /// How much has been recorded.
    pub fn elapsed(&self) -> Duration {
        let samples = self.shared.samples.load(Ordering::Relaxed);
        Duration::from_micros(samples * 1_000_000 / u64::from(MODEL_RATE))
    }

    /// What has been heard so far, and a number that changes whenever it
    /// does. The last words may change as more is heard; the transcript
    /// [`stop`](Self::stop) gives is the final one.
    pub fn live(&self) -> (u64, String) {
        self.shared
            .live
            .lock()
            .map(|live| (live.version(), live.text()))
            .unwrap_or_default()
    }

    /// False once stopped, or when the device failed, the model wouldn't
    /// load, or [`MAX_RECORDING`] was reached; [`stop`](Self::stop) then
    /// gives the transcript of what was recorded.
    pub fn is_recording(&self) -> bool {
        self.shared.recording.load(Ordering::Relaxed)
    }

    /// Stops recording; the receiver yields the full transcript (or error)
    /// once the last chunk is done. Nothing heard comes back as
    /// [`VoiceError::NothingHeard`]; a device that failed partway gives what
    /// was heard before it did.
    pub fn stop(mut self) -> async_channel::Receiver<Result<String, VoiceError>> {
        self.stopped = true;
        self.shared.stop.store(true, Ordering::Relaxed);
        self.result.clone()
    }

    /// Stops and discards; no transcript.
    pub fn cancel(self) {
        // Dropping cancels.
    }
}

impl Drop for Dictation {
    fn drop(&mut self) {
        if !self.stopped {
            self.shared.cancel.store(true, Ordering::Relaxed);
            self.shared.stop.store(true, Ordering::Relaxed);
        }
    }
}

/// The transcriber thread: loads the model, then transcribes each piece as
/// it comes, and the tail between pieces.
// no-prop-test: loads the real model; `run_transcriber` has the tests.
fn transcribe_pieces(
    dir: &Path,
    pieces: &Receiver<Piece>,
    shared: &Shared,
) -> Result<String, VoiceError> {
    let mut engine = match Parakeet::load(dir) {
        Ok(engine) => engine,
        Err(e) => {
            // Recording is pointless now; let the app see it stop.
            shared.stop.store(true, Ordering::Relaxed);
            return Err(e);
        }
    };
    run_transcriber(pieces, shared, |samples, cancel| {
        engine.transcribe(samples, cancel)
    })
}

/// Transcribes each piece with `transcribe` as it comes, and while none
/// is waiting, the tail recorded since; joins the pieces' texts at the end.
fn run_transcriber(
    pieces: &Receiver<Piece>,
    shared: &Shared,
    mut transcribe: impl FnMut(&[f32], &AtomicBool) -> Result<String, VoiceError>,
) -> Result<String, VoiceError> {
    let live = |f: &mut dyn FnMut(&mut LiveText)| {
        if let Ok(mut live) = shared.live.lock() {
            f(&mut live);
        }
    };
    loop {
        if shared.cancel.load(Ordering::Relaxed) {
            return Err(VoiceError::Cancelled);
        }
        let failure = match pieces.recv_timeout(PREVIEW_WAIT) {
            Ok(Piece::Audio(samples)) => {
                let text = transcribe(&samples, &shared.cancel)?;
                live(&mut |live| live.commit(text.clone()));
                continue;
            }
            Ok(Piece::End { failure }) => failure,
            Err(RecvTimeoutError::Timeout) => {
                let tail = shared.tail.lock().ok().and_then(|mut tail| tail.take());
                if let Some(samples) = tail {
                    let text = transcribe(&samples, &shared.cancel)?;
                    live(&mut |live| live.preview(text.clone()));
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                Some("the microphone stopped unexpectedly".to_owned())
            }
        };
        let text = shared
            .live
            .lock()
            .map(|live| live.finished())
            .unwrap_or_default();
        return match (text.is_empty(), failure) {
            (false, _) => Ok(text),
            (true, Some(failure)) => Err(VoiceError::Device(failure)),
            (true, None) => Err(VoiceError::NothingHeard),
        };
    }
}

/// Maps a device error onto [`VoiceError`].
// no-prop-test: a lookup over error kinds.
fn device_error(e: cpal::Error) -> VoiceError {
    match e.kind() {
        cpal::ErrorKind::PermissionDenied => VoiceError::MicrophoneDenied,
        cpal::ErrorKind::DeviceNotAvailable => VoiceError::NoMicrophone,
        _ => VoiceError::Device(e.to_string()),
    }
}

/// Why the device stopped, from its error callback.
type Failure = Arc<Mutex<Option<String>>>;

/// The microphone thread: opens the default input, tells `start` how that
/// went, then records until stopped.
// no-prop-test: needs a microphone; the resampler, meter and planner it uses have the properties.
fn record(shared: &Shared, pieces: &Sender<Piece>, ready: &SyncSender<Result<(), VoiceError>>) {
    let failure: Failure = Arc::new(Mutex::new(None));
    let opened = open_microphone(failure.clone());
    let (stream, mut consumer, rate) = match opened {
        Ok(opened) => opened,
        Err(e) => {
            let _ = ready.send(Err(e));
            let _ = pieces.send(Piece::End { failure: None });
            return;
        }
    };
    let mut resampler = match Resampler16k::new(rate) {
        Ok(resampler) => resampler,
        Err(e) => {
            let _ = ready.send(Err(VoiceError::Device(e)));
            let _ = pieces.send(Piece::End { failure: None });
            return;
        }
    };
    shared.recording.store(true, Ordering::Relaxed);
    let _ = ready.send(Ok(()));

    let cap = MAX_RECORDING.as_secs() * u64::from(MODEL_RATE);
    let mut pending: Vec<f32> =
        Vec::with_capacity(chunk::MAX_SECONDS as usize * MODEL_RATE as usize);
    let mut raw = Vec::new();
    let mut total = 0u64;
    // How long the tail was when it was last handed on.
    let mut handed = 0usize;
    loop {
        thread::sleep(POLL);
        raw.clear();
        collect(&mut consumer, &mut raw);
        let before = pending.len();
        resampler.push(&raw, &mut pending);
        total += (pending.len() - before) as u64;
        if total >= cap {
            let over = (total - cap) as usize;
            pending.truncate(pending.len() - over.min(pending.len()));
            total = cap;
        }
        shared.samples.store(total, Ordering::Relaxed);
        let recent = &pending[pending.len().saturating_sub(LEVEL_WINDOW)..];
        shared
            .level
            .store(meter_level(rms(recent)).to_bits(), Ordering::Relaxed);

        let cut = chunk::plan_cut(&pending, MODEL_RATE);
        if cut.is_some() || preview_due(pending.len(), handed) {
            let Ok(mut tail) = shared.tail.lock() else {
                break;
            };
            if let Some(cut) = cut {
                let rest = pending.split_off(cut);
                let piece = std::mem::replace(&mut pending, rest);
                if pieces.send(Piece::Audio(piece)).is_err() {
                    break;
                }
            }
            *tail = Some(pending.clone());
            handed = pending.len();
        }
        let failed = failure.lock().map(|f| f.is_some()).unwrap_or(true);
        if shared.stop.load(Ordering::Relaxed) || failed || total >= cap {
            break;
        }
    }

    // Stop the device, then take what it recorded since the last poll.
    let _ = stream.pause();
    drop(stream);
    shared.recording.store(false, Ordering::Relaxed);
    shared.level.store(0f32.to_bits(), Ordering::Relaxed);
    if shared.cancel.load(Ordering::Relaxed) {
        let _ = pieces.send(Piece::End { failure: None });
        return;
    }
    if let Ok(mut tail) = shared.tail.lock() {
        *tail = None;
    }
    raw.clear();
    collect(&mut consumer, &mut raw);
    if total < cap {
        let before = pending.len();
        resampler.push(&raw, &mut pending);
        resampler.finish(&mut pending);
        let room = (cap - total) as usize;
        pending.truncate(before + (pending.len() - before).min(room));
        total += (pending.len() - before) as u64;
        shared.samples.store(total, Ordering::Relaxed);
    }
    for range in chunk::split(&pending, MODEL_RATE) {
        let _ = pieces.send(Piece::Audio(pending[range].to_vec()));
    }
    let failure = failure.lock().ok().and_then(|f| f.clone());
    let _ = pieces.send(Piece::End { failure });
}

/// Moves everything waiting in the ring buffer to the end of `raw`.
// no-prop-test: two slices copied; the recording tests go through it.
fn collect(consumer: &mut rtrb::Consumer<f32>, raw: &mut Vec<f32>) {
    let available = consumer.slots();
    if let Ok(chunk) = consumer.read_chunk(available) {
        let (a, b) = chunk.as_slices();
        raw.extend_from_slice(a);
        raw.extend_from_slice(b);
        chunk.commit_all();
    }
}

/// Opens the default input device at its own rate and format, mixing to
/// one channel into a ring buffer. Returns the stream, the buffer's reading
/// end, and the device's rate.
// no-prop-test: needs a microphone.
fn open_microphone(
    failure: Failure,
) -> Result<(cpal::Stream, rtrb::Consumer<f32>, u32), VoiceError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(VoiceError::NoMicrophone)?;
    let supported = device.default_input_config().map_err(device_error)?;
    let rate = supported.sample_rate();
    let channels = usize::from(supported.channels()).max(1);
    let config = supported.config();
    // Two seconds of room: the thread collects every 20 ms.
    let (producer, consumer) = rtrb::RingBuffer::new((rate as usize * 2).max(8_192));

    let on_error = move |e: cpal::Error| {
        // Overruns lose a few samples; the recording goes on.
        if matches!(
            e.kind(),
            cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied
        ) {
            return;
        }
        if let Ok(mut slot) = failure.lock() {
            slot.get_or_insert_with(|| e.to_string());
        }
    };
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, config, channels, producer, on_error),
        SampleFormat::F64 => build::<f64>(&device, config, channels, producer, on_error),
        SampleFormat::I8 => build::<i8>(&device, config, channels, producer, on_error),
        SampleFormat::I16 => build::<i16>(&device, config, channels, producer, on_error),
        SampleFormat::I24 => build::<cpal::I24>(&device, config, channels, producer, on_error),
        SampleFormat::I32 => build::<i32>(&device, config, channels, producer, on_error),
        SampleFormat::U8 => build::<u8>(&device, config, channels, producer, on_error),
        SampleFormat::U16 => build::<u16>(&device, config, channels, producer, on_error),
        SampleFormat::U32 => build::<u32>(&device, config, channels, producer, on_error),
        other => {
            return Err(VoiceError::Device(format!(
                "the microphone sends {other} samples, which Kanso can't read"
            )));
        }
    }
    .map_err(device_error)?;
    stream.play().map_err(device_error)?;
    Ok((stream, consumer, rate))
}

/// Builds an input stream of `T` samples whose callback mixes each frame
/// to one `f32` and pushes it into `producer`, dropping samples when full.
// no-prop-test: needs a microphone; `for_each_mono` has the properties.
fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    mut producer: rtrb::Producer<f32>,
    on_error: impl FnMut(cpal::Error) + Send + 'static,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream::<T, _, _>(
        config,
        move |data: &[T], _| {
            for_each_mono(
                data,
                channels,
                |s| s.to_sample::<f32>(),
                |mono| {
                    let _ = producer.push(mono);
                },
            );
        },
        on_error,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn shared() -> Shared {
        Shared {
            stop: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            recording: AtomicBool::new(false),
            level: AtomicU32::new(0),
            samples: AtomicU64::new(0),
            tail: Mutex::new(None),
            live: Mutex::new(LiveText::default()),
        }
    }

    /// A stand-in for the model: each sample is a word, `w` and its value.
    fn words(samples: &[f32], _: &AtomicBool) -> Result<String, VoiceError> {
        Ok(samples
            .iter()
            .map(|s| format!("w{s}"))
            .collect::<Vec<_>>()
            .join(" "))
    }

    /// What was said shows before it is finished: the tail first, then the
    /// piece that replaces it, and the transcript is the pieces alone.
    #[test]
    fn the_tail_shows_until_its_piece_arrives() {
        let shared = shared();
        let (tx, rx) = mpsc::channel();
        *shared.tail.lock().unwrap() = Some(vec![1.0, 2.0]);
        let mut seen = Vec::new();
        let mut sent = false;
        let result = run_transcriber(&rx, &shared, |samples, cancel| {
            let text = words(samples, cancel);
            if !sent {
                // The tail is being heard: nothing finished yet.
                seen.push(shared.live.lock().unwrap().text());
                tx.send(Piece::Audio(vec![1.0, 2.0, 3.0])).unwrap();
                tx.send(Piece::End { failure: None }).unwrap();
                sent = true;
            }
            text
        });
        assert_eq!(seen, [""]);
        assert_eq!(result.unwrap(), "w1 w2 w3");
        assert_eq!(shared.live.lock().unwrap().text(), "w1 w2 w3");
    }

    #[test]
    fn nothing_heard_and_a_failed_device() {
        let shared = shared();
        let (tx, rx) = mpsc::channel();
        tx.send(Piece::End { failure: None }).unwrap();
        assert!(matches!(
            run_transcriber(&rx, &shared, words),
            Err(VoiceError::NothingHeard)
        ));
        let (tx, rx) = mpsc::channel();
        tx.send(Piece::End {
            failure: Some("gone".into()),
        })
        .unwrap();
        assert!(matches!(
            run_transcriber(&rx, &shared, words),
            Err(VoiceError::Device(_))
        ));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// However tails and pieces interleave, the transcript is the
        /// pieces' text in order, and the text shown never loses a
        /// finished piece.
        #[test]
        fn prop_transcript_is_the_pieces_whatever_the_tails(
            steps in prop::collection::vec((any::<bool>(), prop::collection::vec(0u8..9, 1..4)), 1..12),
        ) {
            let shared = shared();
            let (tx, rx) = mpsc::channel();
            let mut expected = Vec::new();
            for (is_piece, samples) in &steps {
                let samples: Vec<f32> = samples.iter().map(|&s| f32::from(s)).collect();
                if *is_piece {
                    expected.push(words(&samples, &shared.cancel).unwrap());
                    tx.send(Piece::Audio(samples)).unwrap();
                } else {
                    *shared.tail.lock().unwrap() = Some(samples);
                }
            }
            tx.send(Piece::End { failure: None }).unwrap();
            let mut finished_before = String::new();
            let result = run_transcriber(&rx, &shared, |samples, cancel| {
                let live = shared.live.lock().unwrap();
                assert!(live.text().starts_with(&finished_before));
                finished_before = live.finished();
                drop(live);
                words(samples, cancel)
            });
            let joined = crate::text::join_pieces(expected.iter().map(String::as_str));
            if joined.is_empty() {
                prop_assert!(matches!(result, Err(VoiceError::NothingHeard)));
            } else {
                prop_assert_eq!(result.unwrap(), joined);
            }
        }
    }
}
