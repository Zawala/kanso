//! Which notes in the list are selected. One note is the usual case; ⇧-click
//! selects a run of rows, and ⌘A takes every listed note. (⌘-click opens a
//! tab, so only runs and ⌘A select more than one.) The editor shows the
//! focus, the note clicked last.

/// How a row was clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// A plain click: that note alone.
    Plain,
    /// ⇧-click: every row from the anchor to this one.
    Range,
}

/// The selected notes, the note shown in the editor, and the row a ⇧-click
/// measures from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    /// Each selected note once. Empty exactly when nothing is selected.
    ids: Vec<String>,
    /// The note the editor shows: one of `ids`, or `None` when it is empty.
    focus: Option<String>,
    /// Where a run starts.
    anchor: Option<String>,
}

impl Selection {
    /// `id` alone, or nothing.
    pub fn one(id: Option<String>) -> Self {
        Selection {
            ids: id.iter().cloned().collect(),
            focus: id.clone(),
            anchor: id,
        }
    }

    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn focus(&self) -> Option<&str> {
        self.focus.as_deref()
    }

    #[cfg(test)]
    pub fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|selected| selected == id)
    }

    pub fn count(&self) -> usize {
        self.ids.len()
    }

    /// Whether more than one note is selected.
    pub fn is_several(&self) -> bool {
        self.ids.len() > 1
    }

    /// A click on `id`, one of the `listed` notes in their order.
    pub fn click(&mut self, id: &str, how: Click, listed: &[&str]) {
        match how {
            Click::Plain => *self = Selection::one(Some(id.to_owned())),
            Click::Range => {
                let from = self
                    .anchor
                    .as_deref()
                    .and_then(|anchor| listed.iter().position(|l| *l == anchor));
                let to = listed.iter().position(|l| *l == id);
                let (Some(from), Some(to)) = (from, to) else {
                    return self.click(id, Click::Plain, listed);
                };
                let run = from.min(to)..=from.max(to);
                self.ids = listed[run].iter().map(|l| (*l).to_owned()).collect();
                self.focus = Some(id.to_owned());
            }
        }
    }

    /// Every one of the `listed` notes, keeping the focus when it is listed.
    pub fn select_all(&mut self, listed: &[&str]) {
        let Some(first) = listed.first() else {
            return;
        };
        let focus = match self.focus.as_deref() {
            Some(focus) if listed.contains(&focus) => focus.to_owned(),
            _ => (*first).to_owned(),
        };
        self.ids = listed.iter().map(|l| (*l).to_owned()).collect();
        self.anchor = Some(focus.clone());
        self.focus = Some(focus);
    }

    /// Back to the focus alone.
    pub fn collapse(&mut self) {
        *self = Selection::one(self.focus.clone());
    }

    /// Keeps to the notes still `listed` after the list reloads. When the
    /// focus has gone from the list, it is all that stays selected: the
    /// editor keeps showing it, as it did before several could be selected.
    pub fn retain(&mut self, listed: &[&str]) {
        let focus_listed = self
            .focus
            .as_deref()
            .is_some_and(|focus| listed.contains(&focus));
        if !focus_listed {
            return self.collapse();
        }
        self.ids.retain(|id| listed.contains(&id.as_str()));
        if !self
            .anchor
            .as_deref()
            .is_some_and(|anchor| listed.contains(&anchor))
        {
            self.anchor = self.focus.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{Click, Selection};

    /// One thing done to the selection, over rows `0..ROWS` of the list.
    #[derive(Clone, Debug)]
    enum Op {
        Click(usize, Click),
        All,
        Collapse,
        /// The list reloads with only the rows whose bit is set.
        Reload(Vec<bool>),
    }

    const ROWS: usize = 8;

    fn op() -> impl Strategy<Value = Op> {
        let how = prop_oneof![Just(Click::Plain), Just(Click::Range)];
        prop_oneof![
            6 => (0..ROWS, how).prop_map(|(ix, how)| Op::Click(ix, how)),
            1 => Just(Op::All),
            1 => Just(Op::Collapse),
            1 => proptest::collection::vec(any::<bool>(), ROWS).prop_map(Op::Reload),
        ]
    }

    /// The same rules over row numbers and a set, written out again from
    /// their description: what the real selection is checked against.
    #[derive(Clone, Debug, Default)]
    struct Model {
        set: BTreeSet<usize>,
        focus: Option<usize>,
        anchor: Option<usize>,
    }

    impl Model {
        fn one(ix: Option<usize>) -> Model {
            Model {
                set: ix.into_iter().collect(),
                focus: ix,
                anchor: ix,
            }
        }

        /// Does `op` to the model. `then_focus` is the real selection's
        /// focus, checked by the caller to be selected.
        fn apply(&mut self, op: &Op, listed: &[usize], then_focus: Option<usize>) {
            let _ = then_focus;
            match op {
                Op::Click(ix, Click::Plain) => *self = Model::one(Some(*ix)),
                Op::Click(ix, Click::Range) => {
                    let from = self
                        .anchor
                        .and_then(|a| listed.iter().position(|l| *l == a));
                    let to = listed.iter().position(|l| l == ix);
                    match (from, to) {
                        (Some(from), Some(to)) => {
                            self.set = listed[from.min(to)..=from.max(to)]
                                .iter()
                                .copied()
                                .collect();
                            self.focus = Some(*ix);
                        }
                        _ => *self = Model::one(Some(*ix)),
                    }
                }
                Op::All => {
                    if let Some(first) = listed.first() {
                        let focus = self.focus.filter(|f| listed.contains(f)).unwrap_or(*first);
                        self.set = listed.iter().copied().collect();
                        self.focus = Some(focus);
                        self.anchor = Some(focus);
                    }
                }
                Op::Collapse => *self = Model::one(self.focus),
                Op::Reload(_) => match self.focus {
                    Some(focus) if listed.contains(&focus) => {
                        self.set.retain(|ix| listed.contains(ix));
                        if !self.anchor.is_some_and(|a| listed.contains(&a)) {
                            self.anchor = self.focus;
                        }
                    }
                    _ => *self = Model::one(self.focus),
                },
            }
        }
    }

    fn name(ix: usize) -> String {
        format!("n{ix}")
    }

    fn row(id: &str) -> usize {
        id[1..].parse().unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Model: any run of clicks, ⌘A, Escape and reloads selects the same
        /// rows, with the same focus and anchor, as the row-number model.
        /// Invariants: no note is selected twice, and the focus is one of
        /// the selected whenever anything is.
        #[test]
        fn prop_selection_follows_the_model(ops in proptest::collection::vec(op(), 0..40)) {
            let mut listed: Vec<usize> = (0..ROWS).collect();
            let mut real = Selection::default();
            let mut model = Model::default();
            for op in &ops {
                let op = match op {
                    // A click lands on a listed row.
                    Op::Click(ix, how) if !listed.is_empty() => {
                        Op::Click(listed[ix % listed.len()], *how)
                    }
                    Op::Click(..) => continue,
                    Op::Reload(keep) => {
                        listed = (0..ROWS).filter(|ix| keep[*ix]).collect();
                        op.clone()
                    }
                    _ => op.clone(),
                };
                let names: Vec<String> = listed.iter().copied().map(name).collect();
                let refs: Vec<&str> = names.iter().map(String::as_str).collect();
                match &op {
                    Op::Click(ix, how) => real.click(&name(*ix), *how, &refs),
                    Op::All => real.select_all(&refs),
                    Op::Collapse => real.collapse(),
                    Op::Reload(_) => real.retain(&refs),
                }
                model.apply(&op, &listed, real.focus().map(row));

                let selected: BTreeSet<usize> = real.ids().iter().map(|id| row(id)).collect();
                prop_assert_eq!(selected.len(), real.ids().len(), "no note twice");
                prop_assert_eq!(&selected, &model.set);
                prop_assert_eq!(real.focus().map(row), model.focus);
                prop_assert_eq!(real.anchor.as_deref().map(row), model.anchor);
                prop_assert_eq!(real.focus().is_some(), !real.ids().is_empty());
                if let Some(focus) = real.focus() {
                    prop_assert!(real.contains(focus), "the focus is selected");
                }
            }
        }
    }

    #[test]
    fn clicks_select_like_a_file_list() {
        let listed = ["a", "b", "c", "d", "e"];
        let mut s = Selection::one(Some("b".into()));
        s.click("d", Click::Range, &listed);
        assert_eq!(s.ids(), ["b", "c", "d"]);
        assert_eq!(s.focus(), Some("d"));
        // A run measures from the same anchor until something else is clicked.
        s.click("a", Click::Range, &listed);
        assert_eq!(s.ids(), ["a", "b"]);
        s.click("d", Click::Range, &listed);
        assert_eq!(s.ids(), ["b", "c", "d"]);
        s.click("c", Click::Plain, &listed);
        assert_eq!(s.ids(), ["c"]);
        s.select_all(&listed);
        assert_eq!(s.count(), 5);
        assert_eq!(s.focus(), Some("c"));
        s.retain(&["a", "c"]);
        assert_eq!(s.ids(), ["a", "c"]);
        s.retain(&["a"]);
        assert_eq!(s.ids(), ["c"], "the focus left the list: it stays, alone");
        s.collapse();
        assert_eq!(s, Selection::one(Some("c".into())));
    }
}
