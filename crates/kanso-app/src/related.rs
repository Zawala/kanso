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
//!
//! While the model is loaded the store borrows it to turn a search into a
//! vector, so the palette (and the local API's `kanso-search-by-meaning`)
//! also finds notes about what was typed, not only notes with its words.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::{App, Global, Task};
use kanso_store::{Embedder, Store};

pub use crate::model_fetch::percent;
use crate::model_fetch::{HttpFetch, Model, ModelError, ModelFile, download};
use crate::workspace::AppState;

/// The model, as the store names its embeddings: the Hugging Face repository
/// and the revision its files are pinned to.
pub const MODEL_ID: &str = "minishlab/potion-base-8M@bf8b0566";
/// The repository's commit the files below come from.
const REVISION: &str = "bf8b056651a2c21b8d2565580b8569da283cab23";
const REPOSITORY: &str = "minishlab/potion-base-8M";
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

/// The model, kept in the data folder's `models/potion-base-8M`.
pub const MODEL: Model = Model {
    repository: REPOSITORY,
    revision: REVISION,
    folder: "models/potion-base-8M",
    files: &FILES,
};

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

/// The model, loaded from `dir` once every file there checks out.
pub struct StaticEmbedder {
    model: model2vec_rs::model::StaticModel,
    dim: usize,
}

impl StaticEmbedder {
    pub fn load(dir: &Path) -> Result<StaticEmbedder, ModelError> {
        if let Some(file) = MODEL.missing(dir)?.first() {
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
    store.lend_embedder(None);
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
    let dir = MODEL.dir(&state.data_dir);
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
            .spawn(async move { MODEL.missing(&dir) })
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
                    &MODEL,
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
    // Searches by meaning turn their words into a vector with it.
    store.lend_embedder(Some(embedder.clone()));
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
pub(crate) async fn futures_lite_select<T>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_fetch::tests::Memory;
    use proptest::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn the_pinned_files_are_on_their_revision_over_https() {
        for file in FILES {
            let url = MODEL.file_url(file.name);
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
            .map(|f| (MODEL.file_url(f.name), vec![b'x'; f.size as usize]))
            .collect();
        let cancel = AtomicBool::new(false);
        let err = download(
            &Memory::new(fake),
            &MODEL,
            dir.path(),
            &mut |_, _| {},
            &cancel,
        )
        .unwrap_err();
        assert!(matches!(err, ModelError::Checksum("config.json")));
        // Nothing was kept, not even the part file.
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        // A file put there by hand, the right size but other bytes.
        for file in FILES {
            fs::write(dir.path().join(file.name), vec![b'y'; file.size as usize]).unwrap();
        }
        assert_eq!(MODEL.missing(dir.path()).unwrap().len(), 3);
        assert!(matches!(
            StaticEmbedder::load(dir.path()),
            Err(ModelError::Checksum(_))
        ));
    }

    proptest! {
        /// Closer scores never read as further apart.
        #[test]
        fn prop_similarity_words_follow_the_score(a in -1f32..1., b in -1f32..1.) {
            let rank = |s: f32| ["Loosely related", "Related", "Close", "Very close"].iter().position(|w| *w == similarity_label(s)).unwrap();
            if a <= b { prop_assert!(rank(a) <= rank(b)); }
        }
    }
}
