//! What the meaning features answer with.

use crate::RelatedNote;

/// What a search by meaning found.
#[derive(Clone, Debug, PartialEq)]
pub enum MeaningSearch {
    /// Related notes are off: there are no embeddings to search.
    Off,
    /// Related notes are on, but this process has no model to turn the
    /// search into a vector: the app is still loading it, or the store was
    /// opened without the app (`kanso serve`).
    NoModel,
    /// The notes nearest the search, most similar first.
    Found(Vec<RelatedNote>),
}
