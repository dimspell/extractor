//! Shared text-encoding registry for DISPEL® game files.
//!
//! This crate is the **single source of truth** for the three text encodings
//! used across the workspace. It owns:
//!
//! - the spellings accepted in `#[extractor(encoding = "...")]`,
//!   `#[binary_record(string(encoding = "..."))]` and
//!   `#[translatable(encoding = "...")]` attributes (consumed by
//!   `dispel-macros` at compile time),
//! - the canonical labels written to CSV/PO exports, and
//! - the `encoding_rs` mapping used by the runtime helpers in `dispel-core`.
//!
//! Keeping these in one place means a new accepted spelling or a label change
//! only has to be made once. In particular, the derive macros no longer each
//! carry their own — subtly different — copy of the spelling table.

use serde::{Deserialize, Serialize};

/// A text encoding supported by the game file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextEncoding {
    Windows1250,
    EucKr,
    Utf8,
}

impl TextEncoding {
    /// Every encoding, in a stable order (useful for exhaustive tests).
    pub const ALL: [Self; 3] = [Self::Windows1250, Self::EucKr, Self::Utf8];

    /// Canonical label used in CSV / PO exports.
    pub fn label(self) -> &'static str {
        self.spec().label
    }

    /// All spellings accepted in derive attributes and by [`Self::from_spelling`].
    ///
    /// Spellings are compared case-insensitively, so `"euc-kr"`, `"EUC-KR"`
    /// and `"EUC_KR"` all resolve to [`TextEncoding::EucKr`].
    pub fn spellings(self) -> &'static [&'static str] {
        self.spec().spellings
    }

    /// Name of the `encoding_rs` constant, e.g. `WINDOWS_1250`.
    ///
    /// The derive macros use this to emit `encoding_rs::WINDOWS_1250` tokens.
    pub fn encoding_rs_name(self) -> &'static str {
        self.spec().rs_name
    }

    /// Name of the `TextEncoding` variant, e.g. `Windows1250`.
    ///
    /// The `Localizable` derive uses this to emit
    /// `crate::localization::TextEncoding::Windows1250` tokens.
    pub fn variant_name(self) -> &'static str {
        self.spec().variant_name
    }

    /// The `encoding_rs` codec for this encoding.
    pub fn encoding_rs(self) -> &'static encoding_rs::Encoding {
        match self {
            Self::Windows1250 => encoding_rs::WINDOWS_1250,
            Self::EucKr => encoding_rs::EUC_KR,
            Self::Utf8 => encoding_rs::UTF_8,
        }
    }

    /// Resolve any accepted spelling to an encoding.
    pub fn from_spelling(spelling: &str) -> Option<Self> {
        REGISTRY.iter().find_map(|spec| {
            spec.spellings
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(spelling))
                .then_some(spec.kind)
        })
    }

    fn spec(self) -> &'static EncodingSpec {
        match self {
            Self::Windows1250 => &REGISTRY[0],
            Self::EucKr => &REGISTRY[1],
            Self::Utf8 => &REGISTRY[2],
        }
    }
}

struct EncodingSpec {
    kind: TextEncoding,
    label: &'static str,
    spellings: &'static [&'static str],
    /// `encoding_rs` constant name (used by the derive macros).
    rs_name: &'static str,
    /// `TextEncoding` variant name (used by the `Localizable` derive).
    variant_name: &'static str,
}

static REGISTRY: [EncodingSpec; 3] = [
    EncodingSpec {
        kind: TextEncoding::Windows1250,
        label: "WINDOWS-1250",
        spellings: &["WINDOWS-1250", "WINDOWS_1250"],
        rs_name: "WINDOWS_1250",
        variant_name: "Windows1250",
    },
    EncodingSpec {
        kind: TextEncoding::EucKr,
        label: "EUC-KR",
        spellings: &["EUC-KR", "EUC_KR"],
        rs_name: "EUC_KR",
        variant_name: "EucKr",
    },
    EncodingSpec {
        kind: TextEncoding::Utf8,
        label: "UTF-8",
        spellings: &["UTF-8", "UTF_8"],
        rs_name: "UTF_8",
        variant_name: "Utf8",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_accepted_spelling_resolves_to_its_encoding() {
        for encoding in TextEncoding::ALL {
            for spelling in encoding.spellings() {
                assert_eq!(
                    TextEncoding::from_spelling(spelling),
                    Some(encoding),
                    "spelling {spelling:?} should resolve to {encoding:?}"
                );
            }
        }
    }

    #[test]
    fn spellings_are_case_insensitive() {
        assert_eq!(
            TextEncoding::from_spelling("windows-1250"),
            Some(TextEncoding::Windows1250)
        );
        assert_eq!(
            TextEncoding::from_spelling("euc_kr"),
            Some(TextEncoding::EucKr)
        );
        assert_eq!(
            TextEncoding::from_spelling("Utf-8"),
            Some(TextEncoding::Utf8)
        );
    }

    #[test]
    fn unknown_spelling_is_rejected() {
        assert_eq!(TextEncoding::from_spelling("ISO-8859-2"), None);
        assert_eq!(TextEncoding::from_spelling(""), None);
    }

    #[test]
    fn names_and_labels_round_trip() {
        for encoding in TextEncoding::ALL {
            assert_eq!(
                TextEncoding::from_spelling(encoding.label()),
                Some(encoding)
            );
            assert!(!encoding.encoding_rs_name().is_empty());
            assert!(!encoding.variant_name().is_empty());
        }
    }
}
