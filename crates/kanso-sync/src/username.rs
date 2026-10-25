//! Usernames (docs/ACCOUNTS.md › Usernames). Pure; implemented in step 1.
//!
//! The Worker (`cloud/src/usernames.ts`) and this module run the same cases,
//! `cloud/test/fixtures/usernames.json`.

/// Why a username was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum UsernameError {
    /// Fails the rules: after trimming, a character outside U+0021–U+007E,
    /// or not `^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$` after ASCII-lowercasing.
    #[error(
        "use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit"
    )]
    Invalid,
    /// Valid, but reserved; refused at sign-up only.
    #[error("that username is reserved")]
    Reserved,
}

/// Names nobody can sign up with (`reserved_username`). Some are too short
/// to be valid anyway; validity is checked first.
pub const RESERVED_USERNAMES: [&str; 23] = [
    "abuse",
    "admin",
    "administrator",
    "anonymous",
    "api",
    "everyone",
    "guest",
    "help",
    "hostmaster",
    "kanso",
    "me",
    "null",
    "owner",
    "postmaster",
    "root",
    "security",
    "server",
    "support",
    "sync",
    "system",
    "undefined",
    "webmaster",
    "www",
];

/// Normalizes a username, on every route that takes one and on the client,
/// before any lookup, rate-limit key or fake answer:
///
/// 1. Strip leading and trailing U+0009, U+000A, U+000C, U+000D and U+0020
///    (these five only).
/// 2. Refuse any remaining character outside U+0021–U+007E, before
///    lowercasing (JS `toLowerCase` maps U+212A KELVIN SIGN to `k`).
/// 3. ASCII-lowercase, then require `^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$`.
///
/// Never returns [`UsernameError::Reserved`]; see [`check_new_username`].
pub fn normalize_username(input: &str) -> Result<String, UsernameError> {
    let trimmed = input.trim_matches(['\t', '\n', '\u{c}', '\r', ' ']);
    if !trimmed.chars().all(|c| ('\u{21}'..='\u{7e}').contains(&c)) {
        return Err(UsernameError::Invalid);
    }
    let name = trimmed.to_ascii_lowercase();
    let bytes = name.as_bytes();
    let edge = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let valid = (3..=32).contains(&bytes.len())
        && edge(bytes[0])
        && edge(bytes[bytes.len() - 1])
        && bytes.iter().all(|&b| edge(b) || b == b'_' || b == b'-');
    if valid {
        Ok(name)
    } else {
        Err(UsernameError::Invalid)
    }
}

/// Whether a normalized username is reserved.
pub fn is_reserved_username(normalized: &str) -> bool {
    RESERVED_USERNAMES.contains(&normalized)
}

/// The sign-up check: [`normalize_username`], then refuse reserved names.
pub fn check_new_username(input: &str) -> Result<String, UsernameError> {
    let name = normalize_username(input)?;
    if is_reserved_username(&name) {
        Err(UsernameError::Reserved)
    } else {
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::Value;

    /// The crate's folder: cargo's at run time (the phone runners set it to where
    /// they copied the fixtures), else where it was built.
    fn manifest_dir() -> std::path::PathBuf {
        std::env::var_os("CARGO_MANIFEST_DIR")
            .map(Into::into)
            .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
    }

    fn fixture() -> Vec<Value> {
        let path = manifest_dir().join("../../cloud/test/fixtures/usernames.json");
        let text = std::fs::read_to_string(path).expect("read usernames.json");
        let value: Value = serde_json::from_str(&text).expect("parse usernames.json");
        value["cases"].as_array().expect("cases").clone()
    }

    /// Example cases shared with the Worker: every `{input, normalized,
    /// error}` of `cloud/test/fixtures/usernames.json`.
    #[test]
    fn the_shared_username_cases() {
        let cases = fixture();
        assert!(cases.len() >= 60);
        for case in &cases {
            let input = case["input"].as_str().unwrap();
            let normalized = case["normalized"].as_str();
            let error = case["error"].as_str();
            let at = format!("{input:?}");
            assert_eq!(
                normalize_username(input).ok().as_deref(),
                normalized,
                "{at}: normalized"
            );
            let expected = match error {
                None => Ok(normalized.unwrap().to_owned()),
                Some("invalid_username") => Err(UsernameError::Invalid),
                Some("reserved_username") => Err(UsernameError::Reserved),
                Some(other) => panic!("{at}: unknown error {other}"),
            };
            assert_eq!(check_new_username(input), expected, "{at}: sign-up check");
        }
        for name in RESERVED_USERNAMES {
            let listed = cases.iter().any(|c| {
                c["normalized"].as_str() == Some(name) || c["input"].as_str() == Some(name)
            });
            assert!(listed, "reserved name {name} has no case");
        }
    }

    fn gen_input() -> impl Strategy<Value = String> {
        let edge = prop::sample::select(vec![
            "", " ", "\t", "\n", "\u{c}", "\r", "\u{b}", "\u{a0}", "\u{2028}", "\u{feff}",
            "\u{3000}",
        ]);
        let body = prop_oneof![
            "[a-zA-Z0-9_-]{0,34}",
            "[a-zA-Z0-9_.@ -]{0,34}",
            "\\PC{0,10}",
            Just("\u{212a}anso".to_owned()),
        ];
        (edge.clone(), body, edge).prop_map(|(a, b, c)| format!("{a}{b}{c}"))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Idempotence: a normalized name normalizes to itself.
        #[test]
        fn prop_normalizing_is_idempotent(input in gen_input()) {
            if let Ok(name) = normalize_username(&input) {
                prop_assert_eq!(normalize_username(&name), Ok(name.clone()));
            }
        }

        /// Refinement: an accepted name is 3–32 of `[a-z0-9_-]`, starting and
        /// ending with a letter or digit.
        #[test]
        fn prop_accepted_names_match_the_pattern(input in gen_input()) {
            if let Ok(name) = normalize_username(&input) {
                prop_assert!((3..=32).contains(&name.len()));
                prop_assert!(name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-'));
                prop_assert!(!name.starts_with(['_', '-']) && !name.ends_with(['_', '-']));
            }
        }

        /// Metamorphic: case doesn't matter.
        #[test]
        fn prop_case_insensitive(input in gen_input()) {
            prop_assert_eq!(normalize_username(&input.to_ascii_uppercase()), normalize_username(&input.to_ascii_lowercase()));
        }

        /// Invariant: a name with any non-ASCII character left after the five
        /// trimmed ones is refused.
        #[test]
        fn prop_non_ascii_is_refused(input in gen_input()) {
            let trimmed = input.trim_matches(['\t', '\n', '\u{c}', '\r', ' ']);
            if !trimmed.is_ascii() {
                prop_assert_eq!(normalize_username(&input), Err(UsernameError::Invalid));
            }
        }
    }
}
