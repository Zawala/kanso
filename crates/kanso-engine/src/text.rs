//! Offsets into a text in the units an editor counts in. The engine speaks
//! UTF-8 bytes; Flutter's `TextEditingValue` and the web speak UTF-16 code
//! units. These convert, and never land inside a character: an offset
//! between the two halves of a surrogate pair (or past the end of the text)
//! goes to the start of the character (the end of the text).

/// The UTF-8 byte offset of UTF-16 offset `utf16` in `text`, at a
/// character boundary: the start of the character that unit is in. Past the
/// end it is the text's length. Pure.
pub fn utf16_to_byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        let next = units + ch.len_utf16();
        if next > utf16 {
            return byte;
        }
        units = next;
    }
    text.len()
}

/// The UTF-16 offset of UTF-8 byte offset `byte` in `text`; an offset inside
/// a character counts as that character's start, and one past the end as
/// the end. Pure.
pub fn byte_to_utf16(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    text[..byte].chars().map(char::len_utf16).sum()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    #[test]
    fn offsets_convert_across_characters_of_every_width() {
        // 'é' is 2 bytes and 1 unit, '日' 3 and 1, '🙂' 4 and 2.
        let text = "aé日🙂b";
        assert_eq!(utf16_to_byte(text, 0), 0);
        assert_eq!(utf16_to_byte(text, 1), 1);
        assert_eq!(utf16_to_byte(text, 2), 3);
        assert_eq!(utf16_to_byte(text, 3), 6);
        // Inside the pair: the start of the emoji.
        assert_eq!(utf16_to_byte(text, 4), 6);
        assert_eq!(utf16_to_byte(text, 5), 10);
        assert_eq!(utf16_to_byte(text, 6), 11);
        assert_eq!(utf16_to_byte(text, 99), 11);
        assert_eq!(byte_to_utf16(text, 6), 3);
        assert_eq!(byte_to_utf16(text, 10), 5);
        // Inside a character: its start.
        assert_eq!(byte_to_utf16(text, 7), 3);
        assert_eq!(byte_to_utf16(text, 99), 6);
        assert_eq!(utf16_to_byte("", 3), 0);
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Every character boundary converts to a UTF-16 offset and back
        /// to itself.
        #[test]
        fn prop_boundaries_round_trip(text in "\\PC{0,24}") {
            for (byte, _) in text.char_indices().chain([(text.len(), ' ')]) {
                let units = byte_to_utf16(&text, byte);
                prop_assert_eq!(utf16_to_byte(&text, units), byte);
            }
        }

        /// A UTF-16 offset anywhere gives a character boundary, no later
        /// than the offset, and a larger offset never gives a smaller one.
        #[test]
        fn prop_utf16_offsets_floor_to_a_boundary_and_never_go_back(
            text in "\\PC{0,24}",
            a in 0usize..60,
            b in 0usize..60,
        ) {
            let (a, b) = (a.min(b), a.max(b));
            let (low, high) = (utf16_to_byte(&text, a), utf16_to_byte(&text, b));
            prop_assert!(text.is_char_boundary(low) && text.is_char_boundary(high));
            prop_assert!(low <= high && high <= text.len());
            prop_assert!(byte_to_utf16(&text, low) <= a);
        }

        /// A byte offset anywhere (inside a character too) gives a UTF-16
        /// offset that is never past the text and never goes back as the
        /// byte offset grows.
        #[test]
        fn prop_byte_offsets_never_go_back(text in "\\PC{0,24}", a in 0usize..120, b in 0usize..120) {
            let (a, b) = (a.min(b), a.max(b));
            let total: usize = text.chars().map(char::len_utf16).sum();
            prop_assert!(byte_to_utf16(&text, a) <= byte_to_utf16(&text, b));
            prop_assert!(byte_to_utf16(&text, b) <= total);
        }
    }
}
