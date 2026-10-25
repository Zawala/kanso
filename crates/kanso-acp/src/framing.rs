//! Newline-delimited JSON framing: one JSON-RPC message per line, with a cap
//! on how long one line may grow.

/// One line could not be framed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// A line grew past the cap before its newline arrived.
    TooLarge,
    /// The stream ended in the middle of a line.
    Partial,
}

/// Splits bytes as they arrive into complete lines. A message split across
/// reads is joined; several messages in one read come out in order.
#[derive(Debug)]
pub struct LineFramer {
    buf: Vec<u8>,
    max_line: usize,
}

impl LineFramer {
    pub fn new(max_line: usize) -> Self {
        LineFramer {
            buf: Vec::new(),
            max_line,
        }
    }

    /// Adds `bytes` and returns every line they complete, without its
    /// newline. Blank lines are skipped. After an error the framer is spent.
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, FrameError> {
        let mut lines = Vec::new();
        let mut rest = bytes;
        while let Some(end) = rest.iter().position(|&byte| byte == b'\n') {
            let (head, tail) = rest.split_at(end);
            if self.buf.len() + head.len() > self.max_line {
                return Err(FrameError::TooLarge);
            }
            self.buf.extend_from_slice(head);
            let line = std::mem::take(&mut self.buf);
            let line = String::from_utf8_lossy(&line);
            let line = line.trim_end_matches('\r');
            if !line.trim().is_empty() {
                lines.push(line.to_owned());
            }
            rest = &tail[1..];
        }
        if self.buf.len() + rest.len() > self.max_line {
            return Err(FrameError::TooLarge);
        }
        self.buf.extend_from_slice(rest);
        Ok(lines)
    }

    /// The stream ended: anything left is half a message.
    pub fn finish(&self) -> Result<(), FrameError> {
        if self.buf.iter().all(u8::is_ascii_whitespace) {
            Ok(())
        } else {
            Err(FrameError::Partial)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameError, LineFramer};
    use proptest::prelude::*;

    #[test]
    fn a_message_split_at_every_byte_comes_out_whole() {
        let wire = b"{\"jsonrpc\":\"2.0\",\"id\":1}\n";
        for split in 0..=wire.len() {
            let mut framer = LineFramer::new(1024);
            let mut lines = framer.push(&wire[..split]).unwrap();
            lines.extend(framer.push(&wire[split..]).unwrap());
            assert_eq!(lines, vec!["{\"jsonrpc\":\"2.0\",\"id\":1}".to_owned()]);
            framer.finish().unwrap();
        }
    }

    #[test]
    fn two_messages_in_one_read_come_out_in_order() {
        let mut framer = LineFramer::new(1024);
        let lines = framer.push(b"{\"a\":1}\r\n\n{\"b\":2}\n{\"c\"").unwrap();
        assert_eq!(lines, vec!["{\"a\":1}", "{\"b\":2}"]);
        assert_eq!(framer.finish(), Err(FrameError::Partial));
    }

    #[test]
    fn a_line_over_the_cap_is_refused_before_its_newline() {
        let mut framer = LineFramer::new(8);
        assert_eq!(framer.push(b"12345"), Ok(vec![]));
        assert_eq!(framer.push(b"6789"), Err(FrameError::TooLarge));
        let mut framer = LineFramer::new(8);
        assert_eq!(framer.push(b"123456789\n"), Err(FrameError::TooLarge));
        let mut framer = LineFramer::new(8);
        assert_eq!(framer.push(b"12345678\n"), Ok(vec!["12345678".to_owned()]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Messages cut into any reads come out complete, once each, in the
        /// order they were written.
        #[test]
        fn prop_framing_never_loses_or_reorders_messages(
            messages in prop::collection::vec("[a-z0-9{}\":, ]{1,40}", 0..12),
            cuts in prop::collection::vec(0usize..600, 0..20),
        ) {
            let messages: Vec<String> = messages
                .into_iter()
                .filter(|message| !message.trim().is_empty())
                .collect();
            let wire: Vec<u8> = messages
                .iter()
                .flat_map(|message| message.bytes().chain(std::iter::once(b'\n')))
                .collect();
            let mut cuts: Vec<usize> = cuts.into_iter().map(|cut| cut.min(wire.len())).collect();
            cuts.push(0);
            cuts.push(wire.len());
            cuts.sort_unstable();
            let mut framer = LineFramer::new(4096);
            let mut lines = Vec::new();
            for pair in cuts.windows(2) {
                lines.extend(framer.push(&wire[pair[0]..pair[1]]).unwrap());
            }
            prop_assert!(framer.finish().is_ok());
            prop_assert_eq!(lines, messages);
        }
    }
}
