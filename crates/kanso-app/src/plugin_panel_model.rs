//! What a plugin panel's inputs hold, worked out from its tree. Pure.

use std::collections::BTreeMap;

use kanso_plugins::panel_model::Node;
use serde_json::Value;

/// The value a node starts with, if it holds one: an input's text, a
/// toggle's state, a select's chosen option. A select with none chosen
/// holds nothing.
fn own_value(node: &Node) -> Option<(&str, Value)> {
    match node {
        Node::Input { id, value, .. } => Some((id, Value::String(value.clone()))),
        Node::Toggle { id, value, .. } => Some((id, Value::Bool(*value))),
        Node::Select {
            id,
            value: Some(value),
            ..
        } => Some((id, Value::String(value.clone()))),
        _ => None,
    }
}

/// Every input, toggle and select node in the tree, in order.
fn holders(node: &Node, out: &mut Vec<Node>) {
    match node {
        Node::Row { children, .. } | Node::Column { children, .. } => {
            children.iter().for_each(|c| holders(c, out))
        }
        Node::List { items } => items.iter().for_each(|c| holders(c, out)),
        Node::Input { .. } | Node::Toggle { .. } | Node::Select { .. } => out.push(node.clone()),
        _ => {}
    }
}

/// The values for `root`'s inputs: what the person already entered for an
/// ID still in the tree, and the node's own value for any other. IDs no
/// longer in the tree are dropped; nothing is made up.
pub fn seed(root: &Node, entered: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut nodes = Vec::new();
    holders(root, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| {
            let (id, initial) = match own_value(node) {
                Some((id, initial)) => (id, Some(initial)),
                None => match node {
                    Node::Select { id, .. } => (id.as_str(), None),
                    _ => return None,
                },
            };
            entered
                .get(id)
                .cloned()
                .or(initial)
                .map(|value| (id.to_owned(), value))
        })
        .collect()
}

/// The text an input shows for `value`.
pub fn text_of(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    }
}

/// Whether a toggle is on for `value`, or its own `initial` state.
pub fn on_of(value: Option<&Value>, initial: bool) -> bool {
    value.and_then(Value::as_bool).unwrap_or(initial)
}

/// Whether a link may be opened: an `https://` address with a host.
pub fn opens(url: &str) -> bool {
    url.strip_prefix("https://")
        .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::{on_of, opens, seed, text_of};
    use kanso_plugins::panel::inputs;
    use kanso_plugins::panel_model::{Node, TextStyle};
    use proptest::prelude::*;
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    fn input(id: &str, value: &str) -> Node {
        Node::Input {
            id: id.into(),
            label: id.into(),
            value: value.into(),
            placeholder: String::new(),
        }
    }

    fn toggle(id: &str, value: bool) -> Node {
        Node::Toggle {
            id: id.into(),
            label: id.into(),
            value,
        }
    }

    fn column(children: Vec<Node>) -> Node {
        Node::Column {
            children,
            gap: None,
        }
    }

    #[test]
    fn keeps_what_was_entered_for_ids_still_present() {
        let tree = column(vec![input("a", "start"), toggle("b", false)]);
        let entered = BTreeMap::from([("a".to_owned(), json!("typed"))]);
        let out = seed(&tree, &entered);
        assert_eq!(out["a"], json!("typed"));
        assert_eq!(out["b"], json!(false));
    }

    #[test]
    fn drops_ids_that_left_the_tree() {
        let tree = column(vec![input("a", "")]);
        let entered = BTreeMap::from([("gone".to_owned(), json!("x"))]);
        assert!(!seed(&tree, &entered).contains_key("gone"));
    }

    #[test]
    fn a_select_with_nothing_chosen_holds_nothing() {
        let tree = Node::Select {
            id: "s".into(),
            label: "S".into(),
            options: vec!["x".into()],
            value: None,
        };
        assert!(seed(&tree, &BTreeMap::new()).is_empty());
    }

    #[test]
    fn reads_values_and_links() {
        assert_eq!(text_of(Some(&json!("hi"))), "hi");
        assert_eq!(text_of(Some(&json!(3))), "");
        assert!(on_of(None, true));
        assert!(!on_of(Some(&json!(false)), true));
        assert!(opens("https://example.com/x"));
        assert!(!opens("http://example.com"));
        assert!(!opens("https://"));
        assert!(!opens("javascript:alert(1)"));
    }

    fn leaf() -> impl Strategy<Value = Node> {
        prop_oneof![
            ("[a-d]", "[a-z]{0,4}").prop_map(|(id, v)| input(&id, &v)),
            ("[a-d]", any::<bool>()).prop_map(|(id, v)| toggle(&id, v)),
            "[a-z]{0,4}".prop_map(|value| Node::Text {
                value,
                style: TextStyle::Body
            }),
        ]
    }

    proptest! {
        #[test]
        fn kept_ids_are_those_in_both_and_none_are_invented(
            leaves in proptest::collection::vec(leaf(), 0..8),
            entered in proptest::collection::btree_map("[a-f]", "[a-z]{0,4}", 0..6),
        ) {
            let tree = column(leaves);
            let entered: BTreeMap<String, Value> =
                entered.into_iter().map(|(k, v)| (k, json!(v))).collect();
            let out = seed(&tree, &entered);
            let present = inputs(&tree);
            for id in out.keys() {
                prop_assert!(present.contains(&id.as_str()));
            }
            for id in &present {
                prop_assert!(out.contains_key(*id));
                if let Some(kept) = entered.get(*id) {
                    prop_assert_eq!(&out[*id], kept);
                }
            }
            for id in entered.keys() {
                if !present.contains(&id.as_str()) {
                    prop_assert!(!out.contains_key(id));
                }
            }
        }
    }
}
