//! Snapshot states for a plugin's panel in the open inspector: a panel
//! with one of most things a plugin can draw, and the same while a press
//! is running.

use std::collections::BTreeMap;

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_plugins::panel_model::{Node, Panel, TextStyle, Tone};
use kanso_store::Store;
use serde_json::json;

use crate::config::InspectorTab;
use crate::plugins_model::PanelState;
use crate::workspace::{AppState, Workspace};

/// The shots: name (`dark-…` or `light-…`), how to set the window up,
/// and the width of a dialog it opens, if any.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-plugin-panel", panel, None),
    ("light-plugin-panel", panel, None),
    ("dark-plugin-panel-busy", panel_busy, None),
    ("light-plugin-panel-busy", panel_busy, None),
];

/// A weekly review plugin's panel.
fn tree() -> Node {
    let text = |value: &str, style| Node::Text {
        value: value.into(),
        style,
    };
    let badge = |value: &str| Node::Badge {
        value: value.into(),
    };
    let button = |id: &str, label: &str, tone| Node::Button {
        id: id.into(),
        label: label.into(),
        tone,
    };
    Node::Column {
        gap: None,
        children: vec![
            Node::Heading {
                value: "Weekly review".into(),
            },
            text(
                "Collects the notes you changed this week and drafts a summary.",
                TextStyle::Muted,
            ),
            Node::Row {
                gap: None,
                children: vec![badge("12 notes"), badge("3 notebooks"), badge("Draft")],
            },
            Node::Divider,
            Node::Input {
                id: "title".into(),
                label: "Summary title".into(),
                value: "Week 42".into(),
                placeholder: "Title".into(),
            },
            Node::Toggle {
                id: "tasks".into(),
                label: "Include open tasks".into(),
                value: true,
            },
            Node::Select {
                id: "notebook".into(),
                label: "Save into".into(),
                options: vec!["Journal".into(), "Work".into()],
                value: Some("Journal".into()),
            },
            Node::Progress { value: 0.6 },
            text("7 of 12 notes read", TextStyle::Caption),
            Node::Row {
                gap: None,
                children: vec![
                    button("draft", "Draft summary", Tone::Primary),
                    button("reset", "Start over", Tone::Normal),
                ],
            },
        ],
    }
}

/// Shows the panel, with `busy`, in the open inspector.
fn show(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>, busy: bool) {
    cx.update(|cx| {
        let (store, dir) = {
            let app = cx.global::<AppState>();
            (app.store.clone(), app.data_dir.clone())
        };
        crate::plugins::init_for_test(store, &dir, Vec::new(), cx);
        let root = tree();
        let values = crate::plugin_panel_model::seed(&root, &BTreeMap::new());
        cx.global_mut::<crate::plugins::Plugins>().panel = Some(PanelState {
            panel: Panel {
                plugin_id: "dev.example.weekly".into(),
                plugin_name: "Weekly review".into(),
                plugin_version: "1.2.0".into(),
                root,
            },
            developer: true,
            values: if busy {
                let mut values = values;
                values.insert("title".into(), json!("Week 42 summary"));
                values
            } else {
                values
            },
            busy,
        });
        workspace.update(cx, |workspace, cx| {
            workspace.show_inspector(Some(InspectorTab::Outline), cx)
        });
    });
}

/// The panel at rest.
fn panel(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show(cx, workspace, false);
}

/// A press is running: its buttons are off.
fn panel_busy(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    show(cx, workspace, true);
}
