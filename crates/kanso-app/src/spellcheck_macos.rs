//! The system's spelling dictionary on macOS: `NSSpellChecker`, in the
//! language (or languages) chosen in System Settings › Keyboard › Text
//! Input. Its checking may be asked from any thread, which is how
//! `spellcheck` asks it, off the main thread.

use objc2_app_kit::NSSpellChecker;
use objc2_foundation::{NSRange, NSString};

use crate::spellcheck::Dictionary;

/// The shared `NSSpellChecker`.
pub struct SystemDictionary;

// no-prop-test: asks the system's dictionary, which has no oracle here;
// `spellcheck` has the properties, with a word list in its place.
impl Dictionary for SystemDictionary {
    fn knows(&self, word: &str) -> bool {
        let checker = NSSpellChecker::sharedSpellChecker();
        let word = NSString::from_str(word);
        checker.checkSpellingOfString_startingAt(&word, 0).length == 0
    }

    fn guesses(&self, word: &str) -> Vec<String> {
        let checker = NSSpellChecker::sharedSpellChecker();
        let word = NSString::from_str(word);
        let range = NSRange::new(0, word.length());
        checker
            .guessesForWordRange_inString_language_inSpellDocumentWithTag(range, &word, None, 0)
            .map(|guesses| guesses.iter().map(|g| g.to_string()).collect())
            .unwrap_or_default()
    }

    fn learn(&self, word: &str) {
        NSSpellChecker::sharedSpellChecker().learnWord(&NSString::from_str(word));
    }
}
