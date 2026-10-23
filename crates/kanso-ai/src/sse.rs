//! Server-sent events, decoded from bytes so a stream can arrive in any
//! chunks: a network read may end in the middle of a line, or of a UTF-8
//! character.

/// One event's `data` lines, joined by newlines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SseEvent {
    pub data: String,
}

/// A stream's decoder. Feed it bytes as they arrive; it returns the events
/// whose blank line has arrived.
#[derive(Default)]
pub struct SseDecoder {
    buffer: Vec<u8>,
    data: Vec<String>,
}

impl SseDecoder {
    pub fn new() -> SseDecoder {
        SseDecoder::default()
    }

    /// Adds bytes and returns every event that is now complete.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        self.buffer.extend_from_slice(bytes);
        let mut events = Vec::new();
        while let Some(end) = self.buffer.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=end).collect();
            let line = &line[..line.len() - 1];
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if let Some(event) = self.line(line) {
                events.push(event);
            }
        }
        events
    }

    /// Ends the stream. An event whose blank line never arrived still counts,
    /// so a stream that stops right after its last data line loses nothing.
    pub fn finish(&mut self) -> Vec<SseEvent> {
        let line = std::mem::take(&mut self.buffer);
        let mut events = self.push(&line);
        events.extend(self.take_event());
        events
    }

    fn line(&mut self, line: &[u8]) -> Option<SseEvent> {
        if line.is_empty() {
            return self.take_event();
        }
        let text = String::from_utf8_lossy(line);
        let rest = text.strip_prefix("data:")?;
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        self.data.push(rest.to_owned());
        None
    }

    fn take_event(&mut self) -> Option<SseEvent> {
        if self.data.is_empty() {
            return None;
        }
        Some(SseEvent {
            data: std::mem::take(&mut self.data).join("\n"),
        })
    }
}

/// Every event in `text`, as if it all arrived at once.
pub fn parse(text: &str) -> Vec<SseEvent> {
    let mut decoder = SseDecoder::new();
    let mut events = decoder.push(text.as_bytes());
    events.extend(decoder.finish());
    events
}

#[cfg(test)]
mod tests {
    use super::{SseDecoder, parse};
    use proptest::prelude::*;

    /// A short recording with comments, multiple data lines and both line
    /// endings.
    const STREAM: &str = "event: greeting\r\ndata: first\r\n\r\n\
        : keep alive\n\
        data: second\n\
        data: line two\n\n\
        data: third\n\n";

    #[test]
    fn events_join_their_data_lines() {
        let events = parse(STREAM);
        assert_eq!(
            events.iter().map(|e| e.data.as_str()).collect::<Vec<_>>(),
            ["first", "second\nline two", "third"]
        );
    }

    #[test]
    fn a_stream_cut_anywhere_decodes_the_same() {
        let expected = parse(STREAM);
        for cut in 0..=STREAM.len() {
            if !STREAM.is_char_boundary(cut) {
                continue;
            }
            let (head, tail) = STREAM.split_at(cut);
            let mut decoder = SseDecoder::new();
            let mut events = decoder.push(head.as_bytes());
            events.extend(decoder.push(tail.as_bytes()));
            events.extend(decoder.finish());
            assert_eq!(events, expected, "cut at {cut}");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// However the bytes are split into reads, the events and their
        /// data are the same.
        #[test]
        fn prop_any_chunking_decodes_the_same(cuts in prop::collection::vec(0usize..=STREAM.len(), 1..24)) {
            let mut cuts = cuts;
            cuts.sort_unstable();
            let expected = parse(STREAM);
            let mut decoder = SseDecoder::new();
            let mut events = Vec::new();
            let mut start = 0;
            for cut in cuts {
                let cut = cut.max(start).min(STREAM.len());
                if STREAM.is_char_boundary(cut) {
                    events.extend(decoder.push(&STREAM.as_bytes()[start..cut]));
                    start = cut;
                }
            }
            events.extend(decoder.push(&STREAM.as_bytes()[start..]));
            events.extend(decoder.finish());
            prop_assert_eq!(events, expected);
        }
    }
}
