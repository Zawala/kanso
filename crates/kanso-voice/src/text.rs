//! The model's tokens, and turning them back into text.

use std::sync::LazyLock;

use regex::{Captures, Regex};

/// SentencePiece's word boundary, which starts a piece that follows a space.
const WORD_START: char = '\u{2581}';

/// The model's tokens, from `vocab.txt`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vocabulary {
    /// Each token's text, by id, with the word boundary as a space.
    pieces: Vec<String>,
    blank: u32,
}

impl Vocabulary {
    /// Reads `vocab.txt`: one `piece id` per line, ids 0 to n-1 each once.
    /// The blank is `<blk>`, or else the last id.
    pub fn parse(text: &str) -> Result<Vocabulary, String> {
        let mut pieces: Vec<Option<String>> = Vec::new();
        for (number, line) in text.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let (piece, id) = line
                .rsplit_once(' ')
                .ok_or_else(|| format!("vocab.txt line {}: no id", number + 1))?;
            let id: usize = id
                .trim()
                .parse()
                .map_err(|_| format!("vocab.txt line {}: bad id {id:?}", number + 1))?;
            if id >= pieces.len() {
                pieces.resize(id + 1, None);
            }
            if pieces[id].is_some() {
                return Err(format!("vocab.txt: id {id} appears twice"));
            }
            pieces[id] = Some(piece.replace(WORD_START, " "));
        }
        let pieces: Vec<String> = pieces
            .into_iter()
            .enumerate()
            .map(|(id, piece)| piece.ok_or_else(|| format!("vocab.txt: id {id} is missing")))
            .collect::<Result<_, _>>()?;
        if pieces.is_empty() {
            return Err("vocab.txt is empty".into());
        }
        let blank = pieces
            .iter()
            .position(|p| p == "<blk>")
            .unwrap_or(pieces.len() - 1) as u32;
        Ok(Vocabulary { pieces, blank })
    }

    /// How many tokens there are, the blank among them.
    pub fn len(&self) -> usize {
        self.pieces.len()
    }

    /// Whether there are no tokens; never true for a parsed vocabulary.
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// The blank's id.
    pub fn blank(&self) -> u32 {
        self.blank
    }

    /// The text of token `id`, if there is one.
    pub fn piece(&self, id: u32) -> Option<&str> {
        self.pieces.get(id as usize).map(String::as_str)
    }

    /// The text `tokens` spell, as [`detokenize`] joins it. Control tokens
    /// (`<unk>`, `<blk>`, `<|…|>`) and unknown ids are left out.
    pub fn decode(&self, tokens: impl IntoIterator<Item = u32>) -> String {
        let pieces: Vec<&str> = tokens
            .into_iter()
            .filter_map(|id| self.piece(id))
            .filter(|piece| !is_control(piece))
            .collect();
        detokenize(&pieces)
    }
}

/// Whether a piece is a control token rather than text.
fn is_control(piece: &str) -> bool {
    matches!(piece, "<unk>" | "<pad>" | "<blk>")
        || (piece.starts_with("<|") && piece.ends_with("|>"))
}

/// Joins pieces (word boundaries already spaces) into text as onnx-asr
/// does: no space at the start, none before punctuation or at the end, and
/// one before each word. The result is trimmed.
pub fn detokenize(pieces: &[&str]) -> String {
    // onnx-asr's DECODE_SPACE_PATTERN: a space at the start, or one not
    // before a word, goes; one before a word stays a single space.
    static SPACES: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\A\s|\s\B|(\s)\b").expect("valid pattern"));
    let joined = pieces.concat();
    SPACES
        .replace_all(
            &joined,
            |caps: &Captures| if caps.get(1).is_some() { " " } else { "" },
        )
        .trim()
        .to_owned()
}

/// Joins the texts of a recording's pieces with single spaces, skipping
/// empty ones.
pub fn join_pieces<'a>(texts: impl IntoIterator<Item = &'a str>) -> String {
    let mut out = String::new();
    for text in texts.into_iter().map(str::trim).filter(|t| !t.is_empty()) {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn vocab() -> Vocabulary {
        Vocabulary::parse("<unk> 0\n▁Kan 1\nso 2\n▁turns 3\n, 4\n. 5\n▁what 6\n<|en|> 7\n<blk> 8\n")
            .unwrap()
    }

    #[test]
    fn pieces_join_into_words() {
        assert_eq!(vocab().decode([1, 2, 3, 4, 6, 5]), "Kanso turns, what.");
    }

    #[test]
    fn control_tokens_are_left_out() {
        assert_eq!(vocab().decode([0, 7, 1, 2, 8, 99]), "Kanso");
    }

    #[test]
    fn the_blank_is_blk() {
        let v = vocab();
        assert_eq!((v.len(), v.blank()), (9, 8));
        assert_eq!(Vocabulary::parse("a 0\nb 1\n").unwrap().blank(), 1);
    }

    #[test]
    fn broken_vocabularies_are_refused() {
        assert!(Vocabulary::parse("").is_err());
        assert!(Vocabulary::parse("a 0\nb 2\n").is_err());
        assert!(Vocabulary::parse("a 0\nb 0\n").is_err());
        assert!(Vocabulary::parse("a x\n").is_err());
    }

    #[test]
    fn spaces_follow_onnx_asr() {
        assert_eq!(
            detokenize(&[" Hello", ",", " world", " ", "!"]),
            "Hello, world!"
        );
        assert_eq!(detokenize(&[" it", "'s", " 5", "%", " "]), "it's 5%");
        assert_eq!(detokenize(&[]), "");
    }

    #[test]
    fn chunks_join_with_single_spaces() {
        assert_eq!(join_pieces(["one two", "", "  three "]), "one two three");
    }

    proptest! {
        #[test]
        fn words_come_back_with_single_spaces(words in prop::collection::vec("[a-zA-Z0-9]{1,8}", 0..20)) {
            let pieces: Vec<String> = words.iter().map(|w| format!(" {w}")).collect();
            let refs: Vec<&str> = pieces.iter().map(String::as_str).collect();
            prop_assert_eq!(detokenize(&refs), words.join(" "));
        }

        #[test]
        fn detokenized_text_is_trimmed_without_double_spaces(
            pieces in prop::collection::vec(prop::sample::select(vec![" ", " a", "b", ",", ".", " c", "'s", " 1", "?"]), 0..30),
        ) {
            let text = detokenize(&pieces);
            prop_assert!(!text.starts_with(' ') && !text.ends_with(' '));
            prop_assert!(!text.contains("  "));
            prop_assert!(!text.contains(" ,") && !text.contains(" ."));
            let letters = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
            prop_assert_eq!(letters(&text), letters(&pieces.concat()));
        }

        #[test]
        fn a_vocabulary_reads_back_its_pieces(pieces in prop::collection::vec("[a-z▁]{1,6}", 1..50)) {
            let file: String = pieces.iter().enumerate().map(|(i, p)| format!("{p} {i}\n")).collect();
            let v = Vocabulary::parse(&file).unwrap();
            prop_assert_eq!(v.len(), pieces.len());
            prop_assert_eq!(v.blank() as usize, pieces.len() - 1);
            for (i, p) in pieces.iter().enumerate() {
                prop_assert_eq!(v.piece(i as u32).unwrap(), p.replace('▁', " "));
            }
        }

        #[test]
        fn joined_chunks_have_no_doubled_spaces(texts in prop::collection::vec("( ?[a-z]{0,5}){0,4} ?", 0..8)) {
            let joined = join_pieces(texts.iter().map(String::as_str));
            prop_assert!(!joined.starts_with(' ') && !joined.ends_with(' '));
            let words: Vec<&str> = texts.iter().flat_map(|t| t.split_whitespace()).collect();
            prop_assert_eq!(joined.split_whitespace().collect::<Vec<_>>(), words);
        }
    }
}
