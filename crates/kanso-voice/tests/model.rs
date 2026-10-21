//! The real model on a spoken sentence. Needs the downloaded files:
//!
//! ```sh
//! KANSO_VOICE_MODEL_DIR=/path/to/parakeet-tdt-0.6b-v3 cargo test -p kanso-voice --test model -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use kanso_voice::{Parakeet, VoiceError, resample_to_16k};

/// The fixture: macOS `say` reading the sentence below, 16 kHz mono 16-bit.
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/hello.wav");
const SPOKEN: &str = "Kanso turns what you say into text";

fn model_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("KANSO_VOICE_MODEL_DIR")
            .expect("set KANSO_VOICE_MODEL_DIR to the model's folder"),
    )
}

fn read_fixture() -> (Vec<f32>, u32) {
    let mut reader = hound::WavReader::open(FIXTURE).unwrap();
    let spec = reader.spec();
    assert_eq!((spec.channels, spec.bits_per_sample), (1, 16));
    let samples = reader
        .samples::<i16>()
        .map(|s| f32::from(s.unwrap()) / 32_768.0)
        .collect();
    (samples, spec.sample_rate)
}

/// Lowercase words, without punctuation.
fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

#[test]
#[ignore = "needs the downloaded model in KANSO_VOICE_MODEL_DIR"]
fn the_model_transcribes_a_spoken_sentence() {
    let (samples, rate) = read_fixture();
    let audio = resample_to_16k(&samples, rate).unwrap();

    let loading = Instant::now();
    let mut model = Parakeet::load(&model_dir()).unwrap();
    let loaded = loading.elapsed();

    let cancel = AtomicBool::new(false);
    let running = Instant::now();
    let text = model.transcribe_all(&audio, &cancel).unwrap();
    let first = running.elapsed();
    let running = Instant::now();
    let again = model.transcribe_all(&audio, &cancel).unwrap();
    let second = running.elapsed();

    println!("transcript: {text:?}");
    println!(
        "audio {:.2} s; load {:.2} s; first run {:.2} s; second run {:.2} s",
        audio.len() as f64 / 16_000.0,
        loaded.as_secs_f64(),
        first.as_secs_f64(),
        second.as_secs_f64()
    );
    assert_eq!(text, again);
    // The model has never heard of Kanso, and writes what it sounds like
    // ("Can so"); every other word must be there, and "Kanso" by its sound.
    let heard = words(&text);
    let expected = words(SPOKEN);
    for word in &expected[1..] {
        assert!(
            heard.contains(word),
            "heard {heard:?}, expected {expected:?}"
        );
    }
    let letters: String = heard.concat();
    assert!(
        letters.contains("kanso") || letters.contains("canso"),
        "heard {heard:?}, expected {expected:?}"
    );
}

#[test]
#[ignore = "needs the downloaded model in KANSO_VOICE_MODEL_DIR"]
fn silence_and_tiny_pieces_give_no_text() {
    let mut model = Parakeet::load(&model_dir()).unwrap();
    let cancel = AtomicBool::new(false);
    assert_eq!(model.transcribe(&[], &cancel).unwrap(), "");
    assert_eq!(model.transcribe(&[0.0; 800], &cancel).unwrap(), "");
    // A quarter second of silence runs the whole model and says nothing.
    assert_eq!(model.transcribe(&[0.0; 4_000], &cancel).unwrap(), "");
}

#[test]
#[ignore = "needs the downloaded model in KANSO_VOICE_MODEL_DIR"]
fn a_long_recording_is_cut_and_joined() {
    let (samples, rate) = read_fixture();
    let sentence = resample_to_16k(&samples, rate).unwrap();
    // The sentence, a second of silence, twelve times: about 46 s.
    let mut audio = Vec::new();
    for _ in 0..12 {
        audio.extend_from_slice(&sentence);
        audio.extend(std::iter::repeat_n(0.0, 16_000));
    }
    let mut model = Parakeet::load(&model_dir()).unwrap();
    let running = Instant::now();
    let text = model
        .transcribe_all(&audio, &AtomicBool::new(false))
        .unwrap();
    println!(
        "{:.1} s of audio in {:.2} s: {text:?}",
        audio.len() as f64 / 16_000.0,
        running.elapsed().as_secs_f64()
    );
    // Given the very same sentence four or more times in one piece, the
    // model writes one copy fewer; onnx-asr's Python decoder does the same
    // on these files. So: three pieces, each missing at most one.
    let sentences = words(&text)
        .iter()
        .filter(|w| w.as_str() == "turns")
        .count();
    assert!(
        (9..=12).contains(&sentences),
        "heard the sentence {sentences} times"
    );
}

#[test]
#[ignore = "needs the downloaded model in KANSO_VOICE_MODEL_DIR"]
fn cancelling_stops_a_transcription() {
    let (samples, rate) = read_fixture();
    let audio = resample_to_16k(&samples, rate).unwrap();
    let mut model = Parakeet::load(&model_dir()).unwrap();
    let result = model.transcribe(&audio, &AtomicBool::new(true));
    assert!(matches!(result, Err(VoiceError::Cancelled)), "{result:?}");
}

#[test]
fn a_missing_model_is_a_model_error() {
    let result = Parakeet::load(std::path::Path::new("/nonexistent/kanso-voice-model"));
    assert!(matches!(result, Err(VoiceError::Model(_))));
}
