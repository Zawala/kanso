//! A stand-in for the mobile library (#48), to measure what the core weighs on
//! a phone. It is temporary: step 4's `kanso-mobile` replaces it with the
//! real facade.
//!
//! Each export calls into one crate through the entry points the facade will
//! use, with its input taken from a pointer and its result passed through
//! `black_box`, so the linker cannot drop what it calls. Each is wrapped in
//! `catch_unwind`, as the facade's boundary will be, which is what the
//! unwinding build pays for.

use std::hint::black_box;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The text behind `ptr` and `len`, or empty.
///
/// # Safety
/// `ptr` must point to `len` readable bytes, or `len` must be zero.
unsafe fn input<'a>(ptr: *const u8, len: usize) -> &'a str {
    if ptr.is_null() || len == 0 {
        return "";
    }
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, len) }).unwrap_or("")
}

/// Runs `body` as the facade will: a panic becomes zero.
fn guarded(body: impl FnOnce() -> usize) -> usize {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or(0)
}

/// kanso-core: IDs and excerpts.
///
/// # Safety
/// See [`input`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_core(ptr: *const u8, len: usize) -> usize {
    let text = unsafe { input(ptr, len) };
    guarded(|| {
        let id = kanso_core::new_id(kanso_core::DocType::Note);
        black_box(kanso_core::excerpt(&id, text)).len()
    })
}

/// kanso-store: open a library, write a note and search.
///
/// # Safety
/// See [`input`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_store(ptr: *const u8, len: usize) -> usize {
    let path = unsafe { input(ptr, len) };
    guarded(|| {
        let Ok(store) = kanso_store::Store::open(path) else {
            return 0;
        };
        let _ = black_box(store.put(
            kanso_core::DocType::Note,
            serde_json::json!({ "title": "A note", "body": path }),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        ));
        black_box(store.search_summaries(&kanso_store::SearchParams::default()))
            .map(|found| found.len())
            .unwrap_or(0)
    })
}

/// kanso-engine: the highlighter and the engine's text edits.
///
/// # Safety
/// See [`input`].
#[cfg(feature = "full")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_engine(ptr: *const u8, len: usize) -> usize {
    let text = unsafe { input(ptr, len) };
    guarded(|| {
        let mut highlighter = kanso_engine::highlight::Highlighter::new();
        black_box(highlighter.spans(text, 0..text.len())).len()
            + black_box(kanso_engine::highlight::spans(text, 0..text.len())).len()
    })
}

/// kanso-render: maths, a Mermaid diagram and a note as HTML.
///
/// # Safety
/// See [`input`].
#[cfg(feature = "full")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_render(ptr: *const u8, len: usize) -> usize {
    use kanso_render::{Job, Kind, Palette};
    let text = unsafe { input(ptr, len) };
    guarded(|| {
        let mut total = 0;
        for kind in [Kind::InlineMath, Kind::Mermaid, Kind::Excalidraw] {
            let job = Job::new(kind, text, Palette::LIGHT);
            total += black_box(kanso_render::render(&job)).map_or(0, |svg| svg.markup.len());
        }
        total
            + black_box(kanso_render::markdown_safe::to_html_with_options(
                text,
                &markdown::Options::gfm(),
            ))
            .map_or(0, |html| html.len())
    })
}

/// kanso-collab: a shared note's document and its merges.
///
/// # Safety
/// See [`input`].
#[cfg(feature = "full")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_collab(ptr: *const u8, len: usize) -> usize {
    let text = unsafe { input(ptr, len) };
    guarded(|| {
        let doc = kanso_collab::CollabDoc::from_text("Title", text);
        let state = doc.encode_state();
        let loaded = kanso_collab::CollabDoc::load(&state);
        black_box(
            loaded
                .map(|doc| doc.text(kanso_collab::Field::Body).len())
                .unwrap_or(0),
        ) + state.len()
    })
}

/// kanso-import: every kind of library on a path.
///
/// # Safety
/// See [`input`].
#[cfg(feature = "full")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_import(ptr: *const u8, len: usize) -> usize {
    use kanso_import::library::{SourceKind, import_source};
    let path = unsafe { input(ptr, len) };
    guarded(|| {
        let Ok(store) = kanso_store::Store::open(":memory:") else {
            return 0;
        };
        let mut done = 0;
        for kind in [
            SourceKind::Markdown,
            SourceKind::Notion,
            SourceKind::Bear,
            SourceKind::Evernote,
            SourceKind::Joplin,
            SourceKind::StandardNotes,
            SourceKind::AppleJournal,
        ] {
            done += black_box(import_source(
                kind,
                std::path::Path::new(path),
                &store,
                |_| {},
            ))
            .is_ok() as usize;
        }
        done + black_box(kanso_import::import(
            std::path::Path::new(path),
            &store,
            |_| {},
        ))
        .is_ok() as usize
    })
}

/// kanso-sync: the HTTP backend and the sync engine.
///
/// # Safety
/// See [`input`].
#[cfg(feature = "full")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_sync(ptr: *const u8, len: usize) -> usize {
    let server = unsafe { input(ptr, len) };
    guarded(|| {
        let backend = kanso_sync::HttpBackend::new();
        // The engine itself, which needs a signed-in session to run: keep a
        // pointer to it, so it is linked as the facade will link it.
        let engine: fn(
            &kanso_store::Store,
            &dyn kanso_sync::Backend,
            &kanso_sync::Session,
            &kanso_sync::SyncOptions,
        ) -> Result<kanso_sync::SyncReport, kanso_sync::SyncError> = kanso_sync::sync;
        black_box(engine);
        black_box(&backend);
        black_box(kanso_sync::check_endpoint(server)).is_ok() as usize
    })
}

/// kanso-ai: a client and a prompt.
///
/// # Safety
/// See [`input`].
#[cfg(feature = "full")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_size_ai(ptr: *const u8, len: usize) -> usize {
    let question = unsafe { input(ptr, len) };
    guarded(|| {
        let settings = kanso_ai::ClientSettings {
            provider: kanso_ai::Provider::OpenAi,
            base_url: "https://api.openai.com/v1".into(),
            model: "model".into(),
            api_key: Some("key".into()),
            region: String::new(),
        };
        let prompt = kanso_ai::build_prompt(question, &[], kanso_ai::DEFAULT_BUDGET);
        black_box(kanso_ai::client(&settings)).is_ok() as usize + prompt.user.len()
    })
}
