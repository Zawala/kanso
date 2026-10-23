//! Query views drawn in the preview: a `query` fence becomes a table of the
//! notes it lists.
//!
//! Parsing and evaluation are `kanso_core::query_view`'s. The preview only
//! reads the notes (on the background executor, once for every query in the
//! note) and draws the rows; a note's change asks for them again.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash as _, Hasher as _};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use gpui_kit::base::text::{MarkdownExtensions, MarkdownNode, MarkdownParseContext, markdown_ast};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::Date;
use kanso_core::properties::Value;
use kanso_core::query_view::{self, ViewNote, ViewRow, ViewSpec};
use kanso_store::Store;

use crate::preview::LinkSender;
use crate::preview_rich::Rich;
use crate::theme::Colors;
use crate::zoom::px;

/// The custom node a query fence becomes.
pub const QUERY: &str = "kanso-query";

/// A query fence's source and the Markdown it was written as.
#[derive(Clone, Debug)]
pub struct QueryBlock {
    pub source: Arc<str>,
    pub markdown: SharedString,
}

/// The block `node` draws as, if it is a `query` fence.
pub fn parse_block(
    node: &markdown_ast::Node,
    cx: &MarkdownParseContext<'_>,
) -> Option<MarkdownNode> {
    let markdown_ast::Node::Code(code) = node else {
        return None;
    };
    let language = code.lang.as_deref()?.split_whitespace().next()?;
    if !language.eq_ignore_ascii_case("query") {
        return None;
    }
    let source: Arc<str> = code.value.clone().into();
    let written = cx.node_source(node)?;
    Some(
        MarkdownNode::new(
            QUERY,
            QueryBlock {
                source,
                markdown: written.into(),
            },
        )
        .text(code.value.clone())
        .markdown(written),
    )
}

/// Registers the query renderer.
pub fn register(
    extensions: MarkdownExtensions,
    rich: Arc<Rich>,
    links: LinkSender,
) -> MarkdownExtensions {
    extensions.block_renderer(QUERY, move |node, window, cx| {
        render(node, &rich, &links, window, cx)
    })
}

/// Counts changes to the notes, so the next draw reads them again.
static EPOCH: AtomicU64 = AtomicU64::new(0);
type Notes = Option<(u64, Arc<Vec<ViewNote>>)>;
static SNAPSHOT: LazyLock<Mutex<Notes>> = LazyLock::new(Default::default);
/// One read at a time: a note's queries share it.
static READING: LazyLock<Mutex<()>> = LazyLock::new(Default::default);
type CacheKey = (u64, u64, Option<String>);
type Rows = HashMap<CacheKey, Arc<Vec<ViewRow>>>;
static RESULTS: LazyLock<Mutex<Rows>> = LazyLock::new(Default::default);
static WORKING: LazyLock<Mutex<HashSet<CacheKey>>> = LazyLock::new(Default::default);

/// Notes changed: query views read them again on their next draw.
pub fn notes_changed() {
    EPOCH.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut results) = RESULTS.lock() {
        results.clear();
    }
}

/// Every note as a query reads it, read at most once per change. Runs on a
/// background thread.
fn notes(epoch: u64, store: &Store) -> Arc<Vec<ViewNote>> {
    if let Ok(cache) = SNAPSHOT.lock()
        && let Some((at, notes)) = cache.as_ref()
        && *at == epoch
    {
        return notes.clone();
    }
    let _one_at_a_time = READING.lock().ok();
    if let Ok(cache) = SNAPSHOT.lock()
        && let Some((at, notes)) = cache.as_ref()
        && *at == epoch
    {
        return notes.clone();
    }
    let notes = Arc::new(read(store).unwrap_or_default());
    if let Ok(mut cache) = SNAPSHOT.lock() {
        *cache = Some((epoch, notes.clone()));
    }
    notes
}

fn read(store: &Store) -> Option<Vec<ViewNote>> {
    store
        .query_view_notes()
        .ok()
        .map(|rows| rows.into_iter().map(ViewNote::from).collect())
}

/// The rows for `spec`, once they are ready. The first call for a query
/// reads the notes and evaluates on the background executor.
fn rows_for(
    spec: &ViewSpec,
    source: &str,
    rich: &Rich,
    self_id: Option<String>,
    cx: &mut App,
) -> Option<Arc<Vec<ViewRow>>> {
    let epoch = EPOCH.load(Ordering::SeqCst);
    let cache_key = (hash(source, self_id.as_deref()), epoch, self_id);
    if let Ok(results) = RESULTS.lock()
        && let Some(rows) = results.get(&cache_key)
    {
        return Some(rows.clone());
    }
    let Ok(mut working) = WORKING.lock() else {
        return None;
    };
    if !working.insert(cache_key.clone()) {
        return None;
    }
    drop(working);
    let store = rich.store.clone();
    let wake = rich.wake.clone();
    let spec = spec.clone();
    let self_id = cache_key.2.clone();
    cx.background_executor()
        .spawn(async move {
            let notes = notes(epoch, &store);
            let rows = Arc::new(query_view::evaluate(&spec, &notes, self_id.as_deref()));
            if let Ok(mut results) = RESULTS.lock() {
                if results.len() >= 256 {
                    results.clear();
                }
                results.insert(cache_key.clone(), rows);
            }
            if let Ok(mut working) = WORKING.lock() {
                working.remove(&cache_key);
            }
            wake.try_send(()).ok();
        })
        .detach();
    None
}

fn hash(source: &str, self_id: Option<&str>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    self_id.hash(&mut hasher);
    hasher.finish()
}

fn render(
    node: &MarkdownNode,
    rich: &Arc<Rich>,
    links: &LinkSender,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(block) = node.data::<QueryBlock>() else {
        return div().into_any_element();
    };
    let at = node.source_range().map_or(0, |r| r.start) as u64;
    let spec = match query_view::parse(&block.source) {
        Ok(spec) => spec,
        Err(err) => return bad_syntax(block, &err, at, window, cx),
    };
    let self_id = rich.note.lock().ok().and_then(|note| note.clone());
    let Some(rows) = rows_for(&spec, &block.source, rich, self_id, cx) else {
        return waiting(at, cx);
    };
    table(&spec, &rows, at, links, cx)
}

/// A query that can't be read: its source, with a badge saying why.
fn bad_syntax(
    block: &QueryBlock,
    err: &query_view::ParseError,
    at: u64,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let detail = err.message.clone();
    let badge = div()
        .id(ElementId::NamedInteger("kanso-query-badge".into(), at))
        .absolute()
        .top(px(7.))
        .left(px(10.))
        .px(px(7.))
        .py(px(1.))
        .rounded(px(5.))
        .bg(colors.fill)
        .border_1()
        .border_color(colors.divider)
        .text_size(px(11.5))
        .text_color(colors.text_secondary)
        .child("Query")
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(detail.clone()).build(window, cx)
        });
    div()
        .relative()
        .w_full()
        .child(crate::preview::code_view(
            ElementId::NamedInteger("kanso-query-code".into(), at),
            block.markdown.clone(),
            window,
            cx,
        ))
        .child(badge)
        .into_any_element()
}

/// Room held while the notes are read.
fn waiting(at: u64, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    div()
        .id(ElementId::NamedInteger("kanso-query-waiting".into(), at))
        .w_full()
        .h(px(48.))
        .rounded(px(8.))
        .bg(Hsla::from(colors.fill).opacity(0.5))
        .into_any_element()
}

/// The table: a header, then a row for each note, its title first.
fn table(spec: &ViewSpec, rows: &[ViewRow], at: u64, links: &LinkSender, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let labels = spec.labels();
    let mut table = v_flex()
        .id(ElementId::NamedInteger("kanso-query".into(), at))
        .debug_selector(|| "kanso-query".into())
        .w_full()
        .min_w_0()
        .pb(px(6.))
        .child(header(&labels, colors));
    if rows.is_empty() {
        table = table.child(
            div()
                .px(px(8.))
                .py(px(8.))
                .text_size(px(12.5))
                .text_color(colors.text_muted)
                .child("No notes match."),
        );
        return table.into_any_element();
    }
    for (ix, row) in rows.iter().enumerate() {
        let links = links.clone();
        let id = row.id.clone();
        table = table.child(
            h_flex()
                .id(ElementId::NamedInteger("kanso-query-row".into(), ix as u64))
                .debug_selector(move || format!("kanso-query-row-{ix}"))
                .w_full()
                .min_w_0()
                .gap(px(12.))
                .px(px(8.))
                .py(px(5.))
                .rounded(px(5.))
                .hover(|el| el.bg(colors.row_hover))
                .child(
                    div()
                        .id(ElementId::NamedInteger(
                            "kanso-query-title".into(),
                            ix as u64,
                        ))
                        .debug_selector(move || format!("kanso-query-title-{ix}"))
                        .flex_none()
                        .w(relative(0.34))
                        .min_w_0()
                        .truncate()
                        .text_size(px(13.))
                        .text_color(colors.link)
                        .cursor_pointer()
                        .hover(|el| el.text_color(colors.text))
                        .child(row.title.clone())
                        .on_click(move |_, _, _| {
                            links.try_send(id.clone()).ok();
                        }),
                )
                .children(row.cells.iter().map(|cell| {
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(cell.as_ref().map(cell_text).unwrap_or_default())
                })),
        );
    }
    table.into_any_element()
}

fn header(labels: &[String], colors: &Colors) -> AnyElement {
    h_flex()
        .w_full()
        .min_w_0()
        .gap(px(12.))
        .px(px(8.))
        .pb(px(4.))
        .border_b_1()
        .border_color(colors.divider)
        .children(labels.iter().enumerate().map(|(ix, label)| {
            div()
                .when(ix == 0, |el| el.flex_none().w(relative(0.34)))
                .when(ix > 0, |el| el.flex_1())
                .min_w_0()
                .truncate()
                .text_size(px(11.5))
                .text_color(colors.text_muted)
                .child(label.clone())
        }))
        .into_any_element()
}

/// A cell as the app writes it: dates named, a boolean a check or a blank,
/// lists comma-separated.
pub fn cell_text(value: &Value) -> String {
    match value {
        Value::Bool(true) => "✓".into(),
        Value::Bool(false) => String::new(),
        Value::Date(date) => Date::parse_iso(date)
            .map(Date::long_label)
            .unwrap_or_else(|| date.clone()),
        Value::List(items) => items.iter().map(cell_text).collect::<Vec<_>>().join(", "),
        other => other.text(),
    }
}

#[cfg(test)]
mod tests {
    use super::cell_text;
    use kanso_core::properties::Value;

    #[test]
    fn cells_read_as_the_app_writes_them() {
        assert_eq!(cell_text(&Value::Text("Acme".into())), "Acme");
        assert_eq!(cell_text(&Value::Number(4.)), "4");
        assert_eq!(cell_text(&Value::Bool(true)), "✓");
        assert_eq!(cell_text(&Value::Bool(false)), "");
        assert_eq!(
            cell_text(&Value::Date("2026-11-01".into())),
            "Sunday, 1 November 2026"
        );
        assert_eq!(
            cell_text(&Value::List(vec![
                Value::Text("Ana".into()),
                Value::Text("Ben".into()),
            ])),
            "Ana, Ben"
        );
    }
}
