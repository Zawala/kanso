//! Related notes: notes about the same things as the open one, found by
//! comparing embeddings made on this device.
//!
//! Off until turned on in Settings. Turning it on downloads a small static
//! embedding model once (Model2Vec's potion-base-8M, MIT, about 30 MB) from
//! its pinned revision on Hugging Face into the data folder, and every file
//! is checked against the SHA-256 written here before it is kept or
//! loaded. The model only turns words into numbers on this device: nothing
//! about the notes is ever sent anywhere, and the download asks for the
//! model's files and nothing else.
//!
//! Once loaded, notes are embedded in the background a few at a time,
//! notes worked on most recently first, with pauses between batches so the
//! app stays smooth; a note is embedded again only when its words change
//! (the store keeps a hash of them). The store keeps the vectors
//! (`kanso-store/src/embed.rs`), so the inspector's Related tab and the
//! local API's `kanso-related-notes` read them without the model.

use std::fs::{self, File};
use std::io::{self, Read, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::{App, Global, Task};
use kanso_store::{Embedder, Store};
use sha2::{Digest as _, Sha256};

use crate::workspace::AppState;

/// The model, as the store names its embeddings: the Hugging Face repository
/// and the revision its files are pinned to.
pub const MODEL_ID: &str = "minishlab/potion-base-8M@bf8b0566";
/// The repository's commit the files below come from.
const REVISION: &str = "bf8b056651a2c21b8d2565580b8569da283cab23";
const REPOSITORY: &str = "minishlab/potion-base-8M";
/// Where the model is kept, under the data folder.
const MODEL_FOLDER: &str = "models/potion-base-8M";

/// One of the model's files, with the size and SHA-256 it must have.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelFile {
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

/// The model's files at [`REVISION`]. The SHA-256s were taken from the
/// files themselves; for the safetensors file it is also the hash Git LFS
/// records for it.
pub const FILES: [ModelFile; 3] = [
    ModelFile {
        name: "config.json",
        size: 202,
        sha256: "2a6ac0e9aaa356a68a5688070db78fc3a464fefe85d2f06a1905ce3718687553",
    },
    ModelFile {
        name: "tokenizer.json",
        size: 683_666,
        sha256: "e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50",
    },
    ModelFile {
        name: "model.safetensors",
        size: 30_236_760,
        sha256: "f65d0f325faadc1e121c319e2faa41170d3fa07d8c89abd48ca5358d9a223de2",
    },
];

/// How many notes are embedded at once, and the pause after each batch.
const BATCH: usize = 16;
const BATCH_PAUSE: Duration = Duration::from_millis(40);
/// How long notes stay still before changed ones are embedded again.
const SETTLE: Duration = Duration::from_millis(1500);
/// How often the download's progress is shown.
const PROGRESS_TICK: Duration = Duration::from_millis(150);

/// Where related notes stand.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Off,
    /// Fetching the model: bytes so far, of all.
    Downloading { done: u64, total: u64 },
    /// Checking and loading the model.
    Loading,
    /// Embedding notes: how many are done, of all.
    Indexing { done: usize, total: usize },
    /// Every note is embedded.
    Ready { notes: usize },
    /// Something went wrong; Retry starts again.
    Failed(String),
}

/// The app's related-notes state.
#[derive(Default)]
pub struct Related {
    pub status: Status,
    /// Moves on whenever new embeddings are kept, so views ask again.
    pub generation: u64,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
}

impl Global for Related {}

/// The model's folder in the data folder `data_dir`.
pub fn model_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(MODEL_FOLDER)
}

/// The address of a model file at its pinned revision.
pub fn file_url(name: &str) -> String {
    format!("https://huggingface.co/{REPOSITORY}/resolve/{REVISION}/{name}")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether the file at `path` is `file`: its size and SHA-256.
pub fn verify(path: &Path, file: &ModelFile) -> io::Result<bool> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    if meta.len() != file.size {
        return Ok(false);
    }
    let mut hasher = Sha256::new();
    let mut reader = File::open(path)?;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()) == file.sha256)
}

/// Why the model couldn't be fetched or loaded.
#[derive(Debug)]
pub enum ModelError {
    Offline,
    Status(u16),
    /// The bytes aren't the pinned file.
    Checksum(&'static str),
    Cancelled,
    Io(io::Error),
    Load(String),
}

impl std::error::Error for ModelError {}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::Offline => write!(f, "Couldn't reach the download. Check the connection."),
            ModelError::Status(code) => write!(f, "The download answered {code}."),
            ModelError::Checksum(name) => write!(
                f,
                "The downloaded {name} isn't the expected file, so it wasn't kept."
            ),
            ModelError::Cancelled => write!(f, "Cancelled."),
            ModelError::Io(err) => write!(f, "Couldn't save the model: {err}"),
            ModelError::Load(err) => write!(f, "Couldn't load the model: {err}"),
        }
    }
}

impl From<io::Error> for ModelError {
    fn from(err: io::Error) -> Self {
        ModelError::Io(err)
    }
}

/// Opens a download: the body, as it arrives. The app's fetches over
/// HTTPS; tests answer from memory.
pub trait Fetch: Send + Sync {
    fn open(&self, url: &str) -> Result<Box<dyn Read + Send>, ModelError>;
}

/// HTTPS only, every hop of a redirect too, with no cookies or credentials
/// and no identifying headers beyond a plain user agent.
pub struct HttpFetch {
    agent: ureq::Agent,
}

impl Default for HttpFetch {
    fn default() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(5)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .user_agent("Kanso")
            .build()
            .into();
        HttpFetch { agent }
    }
}

impl Fetch for HttpFetch {
    fn open(&self, url: &str) -> Result<Box<dyn Read + Send>, ModelError> {
        let response = self
            .agent
            .get(url)
            .call()
            .map_err(|_| ModelError::Offline)?;
        match response.status().as_u16() {
            200 => Ok(Box::new(response.into_body().into_reader())),
            code => Err(ModelError::Status(code)),
        }
    }
}

/// The files in `dir` that aren't the pinned ones (missing or different).
pub fn missing(dir: &Path) -> io::Result<Vec<ModelFile>> {
    let mut out = Vec::new();
    for file in FILES {
        if !verify(&dir.join(file.name), &file)? {
            out.push(file);
        }
    }
    Ok(out)
}

/// Fetches the model's files that `dir` lacks, each kept only when its
/// size and SHA-256 are the pinned ones. `progress` gets the bytes so far
/// and the total.
pub fn download(
    fetch: &dyn Fetch,
    dir: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<(), ModelError> {
    fs::create_dir_all(dir)?;
    let todo = missing(dir)?;
    let total: u64 = todo.iter().map(|f| f.size).sum();
    let mut done = 0u64;
    progress(0, total);
    for file in todo {
        let to = dir.join(file.name);
        let part = dir.join(format!("{}.part", file.name));
        let result = (|| {
            let mut body = fetch.open(&file_url(file.name))?;
            let mut out = File::create(&part)?;
            let mut hasher = Sha256::new();
            let mut buf = vec![0u8; 64 * 1024];
            let mut got = 0u64;
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Err(ModelError::Cancelled);
                }
                let n = body.read(&mut buf).map_err(|_| ModelError::Offline)?;
                if n == 0 {
                    break;
                }
                got += n as u64;
                if got > file.size {
                    return Err(ModelError::Checksum(file.name));
                }
                hasher.update(&buf[..n]);
                out.write_all(&buf[..n])?;
                progress(done + got, total);
            }
            out.sync_all()?;
            if got != file.size || hex(&hasher.finalize()) != file.sha256 {
                return Err(ModelError::Checksum(file.name));
            }
            fs::rename(&part, &to)?;
            Ok(())
        })();
        if result.is_err() {
            fs::remove_file(&part).ok();
        }
        result?;
        done += file.size;
    }
    Ok(())
}

/// The model, loaded from `dir` once every file there checks out.
pub struct StaticEmbedder {
    model: model2vec_rs::model::StaticModel,
    dim: usize,
}

impl StaticEmbedder {
    pub fn load(dir: &Path) -> Result<StaticEmbedder, ModelError> {
        if let Some(file) = missing(dir)?.first() {
            return Err(ModelError::Checksum(file.name));
        }
        let read = |name: &str| fs::read(dir.join(name));
        // One thread for tokenizing: the indexer runs beside the app, not
        // over it.
        tokenizers::utils::parallelism::set_parallelism(false);
        let model = model2vec_rs::model::StaticModel::from_bytes(
            read("tokenizer.json")?,
            read("model.safetensors")?,
            read("config.json")?,
            None,
        )
        .map_err(|err| ModelError::Load(err.to_string()))?;
        let dim = model.encode_single("kanso").len();
        Ok(StaticEmbedder { model, dim })
    }
}

impl Embedder for StaticEmbedder {
    fn model(&self) -> &str {
        MODEL_ID
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        self.model.encode_with_args(texts, Some(256), BATCH)
    }
}

/// Embeds the notes `store` is waiting on, one batch: returns how many
/// are left and how many notes there are, or `None` when none were
/// waiting.
pub fn embed_batch(store: &Store, embedder: &dyn Embedder) -> kanso_store::Result<(usize, usize)> {
    let queue = store.embedding_queue(embedder.model())?;
    let total = queue.total;
    let waiting = queue.pending.len();
    let batch: Vec<_> = queue.pending.into_iter().take(BATCH).collect();
    let texts: Vec<String> = batch.iter().map(|p| p.text.clone()).collect();
    let vectors = embedder.embed(&texts);
    let rows: Vec<(String, String, Vec<f32>)> = batch
        .into_iter()
        .zip(vectors)
        .map(|(p, v)| (p.id, p.hash, v))
        .collect();
    let kept = rows.len();
    store.put_embeddings(embedder.model(), &rows)?;
    Ok((waiting - kept, total))
}

fn set_status(status: Status, cx: &mut App) {
    let related = cx.default_global::<Related>();
    if related.status != status {
        if matches!(status, Status::Indexing { .. } | Status::Ready { .. }) {
            related.generation += 1;
        }
        related.status = status;
        cx.refresh_windows();
    }
}

/// Starts related notes at launch when they were left on.
pub fn init(cx: &mut App) {
    cx.default_global::<Related>();
    if cx.global::<AppState>().config.discovery.related_notes {
        start(cx);
    }
}

/// Turns related notes on or off, from Settings.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.discovery.related_notes = on);
    if on {
        start(cx);
    } else {
        stop(cx);
    }
}

/// Stops whatever is running and forgets the embeddings; the model stays
/// downloaded for next time.
fn stop(cx: &mut App) {
    let related = cx.default_global::<Related>();
    related.cancel.store(true, Ordering::Relaxed);
    related.task = None;
    let store = cx.global::<AppState>().store.clone();
    if let Err(err) = store.set_embedding_model(None) {
        eprintln!("kanso: couldn't turn related notes off: {err}");
    }
    set_status(Status::Off, cx);
}

/// Starts (or starts again, as Retry does): fetches the model if needed,
/// loads it, then keeps the notes embedded.
pub fn start(cx: &mut App) {
    let state = cx.global::<AppState>();
    let store = state.store.clone();
    let dir = model_dir(&state.data_dir);
    let related = cx.default_global::<Related>();
    related.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    related.cancel = cancel.clone();
    let changes = store.subscribe();
    let task = cx.spawn(async move |cx| {
        let result = run(store, dir, cancel.clone(), changes, cx).await;
        if let Err(err) = result
            && !cancel.load(Ordering::Relaxed)
        {
            cx.update(|cx| set_status(Status::Failed(err.to_string()), cx));
        }
    });
    cx.default_global::<Related>().task = Some(task);
}

async fn run(
    store: Arc<Store>,
    dir: PathBuf,
    cancel: Arc<AtomicBool>,
    changes: async_channel::Receiver<kanso_store::Change>,
    cx: &mut gpui_kit::AsyncApp,
) -> Result<(), ModelError> {
    // The model: fetched once, its progress shown as it comes.
    let needed = {
        let dir = dir.clone();
        cx.background_executor()
            .spawn(async move { missing(&dir) })
            .await?
    };
    if !needed.is_empty() {
        let total: u64 = needed.iter().map(|f| f.size).sum();
        cx.update(|cx| set_status(Status::Downloading { done: 0, total }, cx));
        let progress = Arc::new(Mutex::new((0u64, total)));
        let fetched = {
            let (dir, cancel, progress) = (dir.clone(), cancel.clone(), progress.clone());
            cx.background_executor().spawn(async move {
                download(
                    &HttpFetch::default(),
                    &dir,
                    &mut |done, total| *progress.lock().unwrap() = (done, total),
                    &cancel,
                )
            })
        };
        let mut fetched = std::pin::pin!(fetched);
        loop {
            let tick = cx.background_executor().timer(PROGRESS_TICK);
            match futures_lite_select(&mut fetched, tick).await {
                Some(result) => {
                    result?;
                    break;
                }
                None => {
                    let (done, total) = *progress.lock().unwrap();
                    cx.update(|cx| set_status(Status::Downloading { done, total }, cx));
                }
            }
        }
    }
    cx.update(|cx| set_status(Status::Loading, cx));
    let embedder: Arc<dyn Embedder> = {
        let dir = dir.clone();
        Arc::new(
            cx.background_executor()
                .spawn(async move { StaticEmbedder::load(&dir) })
                .await?,
        )
    };
    store
        .set_embedding_model(Some(MODEL_ID))
        .map_err(|err| ModelError::Load(err.to_string()))?;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        let (store2, embedder2) = (store.clone(), embedder.clone());
        let (left, total) = cx
            .background_executor()
            .spawn(async move { embed_batch(&store2, &*embedder2) })
            .await
            .map_err(|err| ModelError::Load(err.to_string()))?;
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        if left == 0 {
            cx.update(|cx| set_status(Status::Ready { notes: total }, cx));
            // Wait for a note to change, then for the changes to settle.
            loop {
                let Ok(change) = changes.recv().await else {
                    return Ok(());
                };
                if change.id.starts_with("note:") {
                    break;
                }
            }
            loop {
                cx.background_executor().timer(SETTLE).await;
                let mut more = false;
                while let Ok(change) = changes.try_recv() {
                    more |= change.id.starts_with("note:");
                }
                if !more {
                    break;
                }
            }
        } else {
            let done = total.saturating_sub(left);
            cx.update(|cx| set_status(Status::Indexing { done, total }, cx));
            cx.background_executor().timer(BATCH_PAUSE).await;
        }
    }
}

/// Waits for `work` or `tick`, whichever comes first: `Some` with the
/// work's result, or `None` when the tick came first.
async fn futures_lite_select<T>(
    work: &mut std::pin::Pin<&mut impl std::future::Future<Output = T>>,
    tick: impl std::future::Future,
) -> Option<T> {
    use std::task::Poll;
    let mut tick = std::pin::pin!(tick);
    std::future::poll_fn(move |cx| {
        if let Poll::Ready(value) = work.as_mut().poll(cx) {
            return Poll::Ready(Some(value));
        }
        if tick.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}

/// How close two notes' embeddings are, in words, for the Related tab.
pub fn similarity_label(score: f32) -> &'static str {
    match score {
        s if s >= 0.75 => "Very close",
        s if s >= 0.55 => "Close",
        s if s >= 0.35 => "Related",
        _ => "Loosely related",
    }
}

/// "34%" of the model: the download's progress.
pub fn percent(done: u64, total: u64) -> u32 {
    if total == 0 {
        return 100;
    }
    ((done.min(total) as f64 / total as f64) * 100.).floor() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;

    /// Answers from memory, counting what was asked.
    struct Memory(HashMap<String, Vec<u8>>, Mutex<Vec<String>>);

    impl Fetch for Memory {
        fn open(&self, url: &str) -> Result<Box<dyn Read + Send>, ModelError> {
            self.1.lock().unwrap().push(url.to_owned());
            match self.0.get(url) {
                Some(bytes) => Ok(Box::new(io::Cursor::new(bytes.clone()))),
                None => Err(ModelError::Status(404)),
            }
        }
    }

    #[test]
    fn the_pinned_files_are_on_their_revision_over_https() {
        for file in FILES {
            let url = file_url(file.name);
            assert!(url.starts_with("https://huggingface.co/minishlab/potion-base-8M/resolve/"));
            assert!(url.contains(REVISION));
            assert_eq!(file.sha256.len(), 64);
            assert!(file.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
        }
        assert!(MODEL_ID.ends_with(&REVISION[..8]));
    }

    /// A tampered file is never kept and never loaded: the download
    /// refuses bytes that aren't the pinned ones, and a file swapped in the
    /// folder afterwards fails the check before the model loads.
    #[test]
    fn a_tampered_model_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let fake: HashMap<String, Vec<u8>> = FILES
            .iter()
            .map(|f| (file_url(f.name), vec![b'x'; f.size as usize]))
            .collect();
        let fetch = Memory(fake, Mutex::new(Vec::new()));
        let cancel = AtomicBool::new(false);
        let err = download(&fetch, dir.path(), &mut |_, _| {}, &cancel).unwrap_err();
        assert!(matches!(err, ModelError::Checksum("config.json")));
        // Nothing was kept, not even the part file.
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        // A file put there by hand, the right size but other bytes.
        for file in FILES {
            fs::write(dir.path().join(file.name), vec![b'y'; file.size as usize]).unwrap();
        }
        assert_eq!(missing(dir.path()).unwrap().len(), 3);
        assert!(matches!(
            StaticEmbedder::load(dir.path()),
            Err(ModelError::Checksum(_))
        ));
        // Too long a body is cut off before it is all read.
        let long: HashMap<String, Vec<u8>> = FILES
            .iter()
            .map(|f| (file_url(f.name), vec![b'z'; f.size as usize + 10]))
            .collect();
        let fetch = Memory(long, Mutex::new(Vec::new()));
        let empty = tempfile::tempdir().unwrap();
        assert!(matches!(
            download(&fetch, empty.path(), &mut |_, _| {}, &cancel),
            Err(ModelError::Checksum(_))
        ));
        // Offline: a clear error, nothing kept.
        let nothing = Memory(HashMap::new(), Mutex::new(Vec::new()));
        assert!(matches!(
            download(&nothing, empty.path(), &mut |_, _| {}, &cancel),
            Err(ModelError::Status(404))
        ));
        assert_eq!(fs::read_dir(empty.path()).unwrap().count(), 0);
    }

    #[test]
    fn verify_checks_size_and_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f");
        let file = ModelFile {
            name: "f",
            size: 3,
            // SHA-256 of "abc".
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        };
        assert!(!verify(&path, &file).unwrap());
        fs::write(&path, b"abc").unwrap();
        assert!(verify(&path, &file).unwrap());
        fs::write(&path, b"abd").unwrap();
        assert!(!verify(&path, &file).unwrap());
        fs::write(&path, b"abcd").unwrap();
        assert!(!verify(&path, &file).unwrap());
    }

    proptest! {
        /// The progress shown never goes past 100% or backwards.
        #[test]
        fn prop_percent_is_monotonic_and_bounded(done in 0u64..10_000_000, more in 0u64..10_000_000, total in 0u64..10_000_000) {
            let a = percent(done, total);
            prop_assert!(a <= 100);
            prop_assert!(percent(done + more, total) >= a);
        }

        /// Closer scores never read as further apart.
        #[test]
        fn prop_similarity_words_follow_the_score(a in -1f32..1., b in -1f32..1.) {
            let rank = |s: f32| ["Loosely related", "Related", "Close", "Very close"].iter().position(|w| *w == similarity_label(s)).unwrap();
            if a <= b { prop_assert!(rank(a) <= rank(b)); }
        }
    }
}
