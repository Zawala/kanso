//! Which model dictation runs, and the files it needs.
//!
//! The app downloads these files into [`FOLDER`] under its data folder and
//! checks each one's size and SHA-256 before it hands the folder to
//! [`Dictation::start`](crate::Dictation::start). Everything is pinned to one
//! revision of the Hugging Face repository, so a file never changes under a
//! hash.

/// A short name for the model, for settings and logs.
pub const MODEL_ID: &str = "nvidia/parakeet-tdt-0.6b-v3 int8 @8f23f0c";

/// Where the files go, under the app's data folder.
pub const FOLDER: &str = "models/parakeet-tdt-0.6b-v3";

/// The Hugging Face repository the files come from.
pub const REPOSITORY: &str = "istupakov/parakeet-tdt-0.6b-v3-onnx";

/// The repository's commit the files are pinned to.
pub const REVISION: &str = "8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce";

/// Who made the model, and under which license; the app shows this beside
/// the download.
pub const ATTRIBUTION: &str = "Parakeet TDT 0.6B v3 by NVIDIA, CC BY 4.0. ONNX conversion and int8 quantization by istupakov.";

/// The model's license.
pub const LICENSE_URL: &str = "https://creativecommons.org/licenses/by/4.0/";

/// One file of the model, with the size and SHA-256 it must have.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelFile {
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

/// The preprocessor that turns audio into mel features.
pub const PREPROCESSOR: &str = "nemo128.onnx";
/// The Conformer encoder.
pub const ENCODER: &str = "encoder-model.int8.onnx";
/// The prediction network and joint, run once per decoding step.
pub const DECODER_JOINT: &str = "decoder_joint-model.int8.onnx";
/// The tokens, one `piece id` per line.
pub const VOCABULARY: &str = "vocab.txt";
/// The model's settings (type, feature size, subsampling).
pub const CONFIG: &str = "config.json";

/// Every file the model needs, smallest first.
pub const FILES: [ModelFile; 5] = [
    ModelFile {
        name: CONFIG,
        size: 97,
        sha256: "666903c76b9798caf2c210afd4f6cd60b08a8dbf9800ec8d7a3bc0d2148ac466",
    },
    ModelFile {
        name: VOCABULARY,
        size: 93_939,
        sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
    },
    ModelFile {
        name: PREPROCESSOR,
        size: 139_764,
        sha256: "a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f",
    },
    ModelFile {
        name: DECODER_JOINT,
        size: 18_202_004,
        sha256: "eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70",
    },
    ModelFile {
        name: ENCODER,
        size: 652_183_999,
        sha256: "6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09",
    },
];

/// Where `name` downloads from, at the pinned revision. Hugging Face answers
/// with a redirect to its file store, so the client must follow redirects.
pub fn file_url(name: &str) -> String {
    format!("https://huggingface.co/{REPOSITORY}/resolve/{REVISION}/{name}")
}

/// How many bytes the whole model downloads.
pub fn total_size() -> u64 {
    FILES.iter().map(|file| file.size).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_model_is_about_670_megabytes() {
        assert_eq!(total_size(), 670_619_803);
    }

    #[test]
    fn every_file_has_a_lowercase_sha256() {
        for file in FILES {
            assert_eq!(file.sha256.len(), 64, "{}", file.name);
            assert!(
                file.sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            );
        }
    }

    #[test]
    fn urls_point_at_the_pinned_revision() {
        assert_eq!(
            file_url("vocab.txt"),
            "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/vocab.txt"
        );
        assert!(MODEL_ID.ends_with(&REVISION[..7]));
    }

    proptest! {
        #[test]
        fn a_url_ends_with_its_file_name(name in "[A-Za-z0-9._-]{1,40}") {
            let url = file_url(&name);
            prop_assert!(url.starts_with("https://huggingface.co/"));
            prop_assert!(url.contains(REVISION));
            let tail = format!("/{name}");
            prop_assert!(url.ends_with(&tail));
        }
    }
}
