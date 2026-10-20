//! Text in images: the words in image attachments, read on this device so
//! the app's search finds a note by what its pictures say.
//!
//! On macOS, Apple's Vision framework reads them (`VNRecognizeTextRequest`,
//! accurate, with language correction), in the background, one image at a
//! time: images attached since, or never read, or whose bytes changed. The
//! text goes into the file's `kansoOcr` (`kanso-store/src/ocr.rs`), which
//! syncs sealed with the file, so other devices search it too without
//! reading the image again. Windows and Linux have no reader: they skip it
//! and still search text another device read.
//!
//! The reader is behind [`TextReader`], so tests use a fake.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::{App, Global, Task};
use kanso_store::Store;

use crate::workspace::AppState;

/// The pause between images, so reading many never crowds the app.
const IMAGE_PAUSE: Duration = Duration::from_millis(120);
/// How long attachments stay still before new ones are read.
const SETTLE: Duration = Duration::from_millis(2000);

/// Reads the words in an image.
pub trait TextReader: Send + Sync {
    /// Names the reader in the file's `kansoOcr`.
    fn engine(&self) -> &'static str;
    /// The image's words, line by line; empty when it has none.
    fn read(&self, image: &[u8]) -> Result<String, String>;
}

/// The system's reader, where there is one.
pub fn system_reader() -> Option<Arc<dyn TextReader>> {
    #[cfg(target_os = "macos")]
    {
        Some(Arc::new(vision::VisionReader))
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Whether this device can read images at all.
pub fn supported() -> bool {
    cfg!(target_os = "macos")
}

/// Where reading stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Off,
    /// Reading: how many images are left.
    Reading { left: usize },
    /// Every image is read.
    Idle,
}

#[derive(Default)]
pub struct ImageText {
    pub status: Status,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
}

impl Global for ImageText {}

/// Reads the next unread image, if there is one: returns how many were
/// waiting (this one included), 0 when none. An image the reader can't
/// read is kept as having no words, so it isn't tried again until its
/// bytes change.
pub fn read_next(store: &Store, reader: &dyn TextReader) -> kanso_store::Result<usize> {
    let waiting = store.unread_images(64)?;
    let Some(image) = waiting.first() else {
        return Ok(0);
    };
    let text = match store.attachment(&image.id)? {
        Some((_, attachment)) => reader.read(&attachment.data).unwrap_or_else(|err| {
            eprintln!("kanso: couldn't read the text in {}: {err}", image.id);
            String::new()
        }),
        None => String::new(),
    };
    store.set_image_text(&image.id, &image.md5, text.trim(), reader.engine())?;
    Ok(waiting.len())
}

fn set_status(status: Status, cx: &mut App) {
    let state = cx.default_global::<ImageText>();
    if state.status != status {
        state.status = status;
        cx.refresh_windows();
    }
}

/// Starts reading at launch when it is on and this device can.
pub fn init(cx: &mut App) {
    cx.default_global::<ImageText>();
    if cx.global::<AppState>().config.discovery.image_text {
        start(cx);
    }
}

/// Turns reading on or off, from Settings.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.discovery.image_text = on);
    if on {
        start(cx);
    } else {
        let state = cx.default_global::<ImageText>();
        state.cancel.store(true, Ordering::Relaxed);
        state.task = None;
        set_status(Status::Off, cx);
    }
}

fn start(cx: &mut App) {
    let Some(reader) = system_reader() else {
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let changes = store.subscribe();
    let state = cx.default_global::<ImageText>();
    state.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    state.cancel = cancel.clone();
    let task = cx.spawn(async move |cx| {
        loop {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let (store2, reader2) = (store.clone(), reader.clone());
            let waiting = cx
                .background_executor()
                .spawn(async move { read_next(&store2, &*reader2) })
                .await;
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            match waiting {
                Ok(0) => {
                    cx.update(|cx| set_status(Status::Idle, cx));
                    // Wait for a file to change, then for things to settle.
                    loop {
                        let Ok(change) = changes.recv().await else {
                            return;
                        };
                        if change.id.starts_with("file:") && !change.deleted {
                            break;
                        }
                    }
                    cx.background_executor().timer(SETTLE).await;
                    while changes.try_recv().is_ok() {}
                }
                Ok(n) => {
                    cx.update(|cx| set_status(Status::Reading { left: n - 1 }, cx));
                    cx.background_executor().timer(IMAGE_PAUSE).await;
                }
                Err(err) => {
                    eprintln!("kanso: reading text in images stopped: {err}");
                    cx.update(|cx| set_status(Status::Idle, cx));
                    return;
                }
            }
        }
    });
    let state = cx.default_global::<ImageText>();
    state.task = Some(task);
    if state.status == Status::Off {
        state.status = Status::Idle;
    }
}

#[cfg(target_os = "macos")]
mod vision {
    //! Apple's Vision framework: `VNRecognizeTextRequest` on the image's
    //! bytes, accurate, with language correction.

    use objc2::AnyThread as _;
    use objc2::rc::{Retained, autoreleasepool};
    use objc2_foundation::{NSArray, NSData, NSDictionary};
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
    };

    pub struct VisionReader;

    impl super::TextReader for VisionReader {
        fn engine(&self) -> &'static str {
            "vision"
        }

        fn read(&self, image: &[u8]) -> Result<String, String> {
            autoreleasepool(|_| {
                let data = NSData::with_bytes(image);
                let handler = VNImageRequestHandler::initWithData_options(
                    VNImageRequestHandler::alloc(),
                    &data,
                    &NSDictionary::new(),
                );
                let request = VNRecognizeTextRequest::new();
                request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
                request.setUsesLanguageCorrection(true);
                let as_request: Retained<VNRequest> =
                    Retained::into_super(Retained::into_super(request.clone()));
                let requests = NSArray::from_retained_slice(&[as_request]);
                handler
                    .performRequests_error(&requests)
                    .map_err(|err| err.localizedDescription().to_string())?;
                let mut lines = Vec::new();
                if let Some(results) = request.results() {
                    for observation in results.iter() {
                        if let Some(best) = observation.topCandidates(1).firstObject() {
                            lines.push(best.string().to_string());
                        }
                    }
                }
                Ok(lines.join("\n"))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use serde_json::json;
    use std::sync::Mutex;

    /// Says what it was given, and counts.
    struct Fake(Mutex<usize>, bool);

    impl TextReader for Fake {
        fn engine(&self) -> &'static str {
            "fake"
        }
        fn read(&self, image: &[u8]) -> Result<String, String> {
            *self.0.lock().unwrap() += 1;
            if self.1 {
                return Err("unreadable".into());
            }
            Ok(format!("words of {}", String::from_utf8_lossy(image)))
        }
    }

    fn attach(store: &Store, bytes: &[u8], content_type: &str) -> String {
        store
            .put(
                DocType::File,
                json!({
                    "name": "a",
                    "contentType": content_type,
                    "_attachments": {"index": {
                        "content_type": content_type,
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                    }},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    #[test]
    fn images_are_read_once_and_again_when_they_change() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let png = attach(&store, b"one", "image/png");
        attach(&store, b"<svg/>", "image/svg+xml");
        let reader = Fake(Mutex::new(0), false);
        assert_eq!(read_next(&store, &reader).unwrap(), 1);
        assert_eq!(read_next(&store, &reader).unwrap(), 0);
        assert_eq!(*reader.0.lock().unwrap(), 1);
        assert_eq!(
            store.image_text(&png).unwrap().as_deref(),
            Some("words of one")
        );
        // New bytes: read again.
        let rev = store.get(&png).unwrap().unwrap().rev;
        store
            .put(
                DocType::File,
                json!({"_id": png, "_rev": rev, "_attachments": {"index": {
                    "content_type": "image/png",
                    "data": base64::engine::general_purpose::STANDARD.encode(b"two"),
                }}}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(read_next(&store, &reader).unwrap(), 1);
        assert_eq!(
            store.image_text(&png).unwrap().as_deref(),
            Some("words of two")
        );
    }

    #[test]
    fn an_unreadable_image_is_not_tried_forever() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let png = attach(&store, b"x", "image/jpeg");
        let reader = Fake(Mutex::new(0), true);
        assert_eq!(read_next(&store, &reader).unwrap(), 1);
        assert_eq!(read_next(&store, &reader).unwrap(), 0);
        assert_eq!(store.image_text(&png).unwrap().as_deref(), Some(""));
    }

    /// "KANSO" in big block letters, black on white, as a PNG.
    #[cfg(target_os = "macos")]
    fn kanso_png() -> Vec<u8> {
        const GLYPHS: [[&str; 7]; 5] = [
            [
                "X...X", "X..X.", "X.X..", "XX...", "X.X..", "X..X.", "X...X",
            ],
            [
                ".XXX.", "X...X", "X...X", "XXXXX", "X...X", "X...X", "X...X",
            ],
            [
                "X...X", "XX..X", "X.X.X", "X..XX", "X...X", "X...X", "X...X",
            ],
            [
                ".XXXX", "X....", "X....", ".XXX.", "....X", "....X", "XXXX.",
            ],
            [
                ".XXX.", "X...X", "X...X", "X...X", "X...X", "X...X", ".XXX.",
            ],
        ];
        let scale = 12u32;
        let (w, h) = ((5 * 6 + 4) * scale, 11 * scale);
        let image = image::RgbImage::from_fn(w, h, |x, y| {
            let (cx, cy) = (x / scale, y / scale);
            let ink = (2..9).contains(&cy)
                && cx >= 2
                && (cx - 2) % 6 < 5
                && ((cx - 2) / 6) < 5
                && GLYPHS[((cx - 2) / 6) as usize][(cy - 2) as usize].as_bytes()
                    [((cx - 2) % 6) as usize]
                    == b'X';
            if ink {
                image::Rgb([0, 0, 0])
            } else {
                image::Rgb([255, 255, 255])
            }
        });
        let mut png = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        png
    }

    /// Reads a real picture with Vision.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "uses the system's Vision framework; run on demand"]
    fn vision_reads_words() {
        let text = vision::VisionReader.read(&kanso_png()).unwrap();
        assert!(text.to_lowercase().contains("kanso"), "read {text:?}");
    }
}
