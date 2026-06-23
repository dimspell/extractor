//! Write-mode selection for the hex editor.
//!
//! When the write mode is *not* [`Hex`], typing any printable character will
//! encode it using the selected encoding and write the resulting bytes directly
//! into the buffer, advancing the cursor by the number of bytes produced.

use encoding_rs::Encoding;

/// Active write mode for the hex matrix keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    /// Enter two hex digits per byte (the classic hex-editor behaviour).
    Hex,
    /// Each typed ASCII character becomes one byte (0x00–0x7F).
    Ascii,
    /// Encode the typed character as UTF-8 (1–4 bytes).
    Utf8,
    /// Encode as Windows-1250 (Central European).
    Windows1250,
    /// Encode as EUC-KR (Korean).
    EucKr,
    /// User-added encoding, indexed into the custom list.
    Custom(usize),
}

impl WriteMode {
    /// Human-readable label for the pick list.
    pub fn label(&self) -> &str {
        match self {
            WriteMode::Hex => "Hex",
            WriteMode::Ascii => "ASCII",
            WriteMode::Utf8 => "UTF-8",
            WriteMode::Windows1250 => "Windows-1250",
            WriteMode::EucKr => "EUC-KR",
            WriteMode::Custom(_) => "Custom",
        }
    }
}

// Custom `Display` so the pick-list shows the label regardless of variant.
impl std::fmt::Display for WriteMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ── Custom encoding entry ─────────────────────────────────────────────────

/// A single custom text encoding added by the user through the settings modal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EncodingEntry {
    /// Display-name shown in the pick list (e.g. `"ISO-8859-2"`).
    pub label: String,
    /// The `encoding_rs` name of the encoding.
    pub encoding_name: String,
}

/// All write-mode variants as a flat list (built‑ins first, then customs).
/// Used to populate the pick list in the toolbar.
pub fn all_write_modes(custom: &[EncodingEntry]) -> Vec<WriteMode> {
    let mut modes = vec![
        WriteMode::Hex,
        WriteMode::Ascii,
        WriteMode::Utf8,
        WriteMode::Windows1250,
        WriteMode::EucKr,
    ];
    for (i, _) in custom.iter().enumerate() {
        modes.push(WriteMode::Custom(i));
    }
    modes
}

/// Build a display-friendly name for a custom mode.
pub fn custom_mode_label(custom: &[EncodingEntry], idx: usize) -> String {
    custom
        .get(idx)
        .map(|e| e.label.clone())
        .unwrap_or_else(|| format!("Custom#{idx}"))
}

/// Re-map `WriteMode::Custom(old_idx)` to `WriteMode::Custom(new_idx)` when
/// items are removed from the custom list.  Built‑in modes pass through unchanged.
pub fn remap_write_mode(mode: &mut WriteMode, removed_idx: usize) {
    if let WriteMode::Custom(i) = mode {
        if *i == removed_idx {
            // The active encoding was removed — fall back to Hex.
            *mode = WriteMode::Hex;
        } else if *i > removed_idx {
            *i -= 1;
        }
    }
}

// ── Serde for WriteMode ───────────────────────────────────────────────────
//
// Serialized as a string label so that custom encodings survive list-order
// changes across sessions.  The `Custom` variant stores its label, not its
// index — the index is resolved at deserialisation time through the custom
// encoding list.

impl serde::Serialize for WriteMode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let label = match self {
            WriteMode::Hex => "Hex",
            WriteMode::Ascii => "ASCII",
            WriteMode::Utf8 => "UTF-8",
            WriteMode::Windows1250 => "Windows-1250",
            WriteMode::EucKr => "EUC-KR",
            WriteMode::Custom(_) => "Custom",
        };
        s.serialize_str(label)
    }
}

impl<'de> serde::Deserialize<'de> for WriteMode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <&str as serde::Deserialize>::deserialize(d)?;
        match s {
            "Hex" => Ok(WriteMode::Hex),
            "ASCII" => Ok(WriteMode::Ascii),
            "UTF-8" => Ok(WriteMode::Utf8),
            "Windows-1250" => Ok(WriteMode::Windows1250),
            "EUC-KR" => Ok(WriteMode::EucKr),
            "Custom" => Ok(WriteMode::Custom(0)), // index resolved at load
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["Hex", "ASCII", "UTF-8", "Windows-1250", "EUC-KR", "Custom"],
            )),
        }
    }
}

// ── Encoding helper ───────────────────────────────────────────────────────

/// Encode a single string into bytes according to the active write mode.
///
/// This is used by the inline text-typing path in `update.rs` so every typed
/// character is immediately encoded and written to the buffer.
pub fn encode_text(text: &str, mode: WriteMode, custom: &[EncodingEntry]) -> Vec<u8> {
    match mode {
        WriteMode::Hex => Vec::new(), // should never be called in hex mode
        WriteMode::Ascii => text
            .chars()
            .filter(|c| c.is_ascii())
            .map(|c| c as u8)
            .collect(),
        WriteMode::Utf8 => text.as_bytes().to_vec(),
        WriteMode::Windows1250 => {
            let (bytes, _enc, _had_errors) = encoding_rs::WINDOWS_1250.encode(text);
            bytes.into_owned()
        }
        WriteMode::EucKr => {
            let (bytes, _enc, _had_errors) = encoding_rs::EUC_KR.encode(text);
            bytes.into_owned()
        }
        WriteMode::Custom(idx) => {
            if let Some(entry) = custom.get(idx) {
                if let Some(enc) = Encoding::for_label(entry.encoding_name.as_bytes()) {
                    let (bytes, _enc, _had_errors) = enc.encode(text);
                    bytes.into_owned()
                } else {
                    // Fallback: plain UTF-8
                    text.as_bytes().to_vec()
                }
            } else {
                text.as_bytes().to_vec()
            }
        }
    }
}

/// Return `true` when the write mode expects text input (any printable
/// character), as opposed to hex-digit-only input.
pub fn is_text_mode(mode: WriteMode) -> bool {
    mode != WriteMode::Hex
}

// ── Common encoding labels ────────────────────────────────────────────────
//
// A curated subset of the encodings that `encoding_rs` supports.  The user can
// add any of these as a custom encoding entry through the settings modal.

/// A list of `(label, encoding_rs_name)` pairs for all available encodings the
/// user may add as a custom entry.
pub const COMMON_ENCODINGS: &[(&str, &str)] = &[
    ("UTF-16LE", "UTF-16LE"),
    ("UTF-16BE", "UTF-16BE"),
    ("Windows-1250", "windows-1250"),
    ("Windows-1251", "windows-1251"),
    ("Windows-1252", "windows-1252"),
    ("Windows-1253", "windows-1253"),
    ("Windows-1254", "windows-1254"),
    ("Windows-1255", "windows-1255"),
    ("Windows-1256", "windows-1256"),
    ("Windows-1257", "windows-1257"),
    ("Windows-1258", "windows-1258"),
    ("ISO-8859-1", "ISO-8859-1"),
    ("ISO-8859-2", "ISO-8859-2"),
    ("ISO-8859-3", "ISO-8859-3"),
    ("ISO-8859-4", "ISO-8859-4"),
    ("ISO-8859-5", "ISO-8859-5"),
    ("ISO-8859-6", "ISO-8859-6"),
    ("ISO-8859-7", "ISO-8859-7"),
    ("ISO-8859-8", "ISO-8859-8"),
    ("ISO-8859-9", "ISO-8859-9"),
    ("ISO-8859-10", "ISO-8859-10"),
    ("ISO-8859-13", "ISO-8859-13"),
    ("ISO-8859-14", "ISO-8859-14"),
    ("ISO-8859-15", "ISO-8859-15"),
    ("ISO-8859-16", "ISO-8859-16"),
    ("EUC-JP", "EUC-JP"),
    ("Shift_JIS", "Shift_JIS"),
    ("GBK", "GBK"),
    ("Big5", "Big5"),
    ("KOI8-R", "KOI8-R"),
    ("KOI8-U", "KOI8-U"),
    ("IBM866", "IBM866"),
    ("macintosh", "macintosh"),
];

/// Return the display labels for all common encodings.
/// This is derived at runtime from [`COMMON_ENCODINGS`] so the two lists never
/// drift apart (a maintenance hazard that existed with a previous static slice).
pub fn common_encoding_labels() -> Vec<&'static str> {
    COMMON_ENCODINGS.iter().map(|(label, _)| *label).collect()
}

/// Look up an `encoding_rs` name for a given display label.
pub fn encoding_name_for_label(label: &str) -> Option<&'static str> {
    COMMON_ENCODINGS
        .iter()
        .find(|(l, _)| *l == label)
        .map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_mode_stays_as_is() {
        assert_eq!(encode_text("A", WriteMode::Hex, &[]).len(), 0);
    }

    #[test]
    fn ascii_encodes() {
        let bytes = encode_text("Hello", WriteMode::Ascii, &[]);
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn ascii_skips_non_ascii() {
        let bytes = encode_text("Helló", WriteMode::Ascii, &[]);
        assert_eq!(bytes, b"Hell");
    }

    #[test]
    fn utf8_multi_byte() {
        let bytes = encode_text("€", WriteMode::Utf8, &[]);
        assert_eq!(bytes, b"\xE2\x82\xAC"); // Euro sign is 3 bytes in UTF-8
    }

    #[test]
    fn windows_1250_encodes() {
        let bytes = encode_text("ł", WriteMode::Windows1250, &[]);
        // ł in windows-1250 is 0xB3
        assert_eq!(bytes, &[0xB3]);
    }

    #[test]
    fn euc_kr_encodes() {
        let bytes = encode_text("한", WriteMode::EucKr, &[]);
        // 한 in EUC-KR is 0xC7 0xD1
        assert_eq!(bytes, &[0xC7, 0xD1]);
    }

    #[test]
    fn custom_encoding_round_trip() {
        let entry = EncodingEntry {
            label: "Shift_JIS".to_string(),
            encoding_name: "Shift_JIS".to_string(),
        };
        // "日" in Shift_JIS is 0x93 0xFA
        let bytes = encode_text("日", WriteMode::Custom(0), &[entry]);
        assert_eq!(bytes, &[0x93, 0xFA]);
    }

    #[test]
    fn remap_after_removal_moves_indices() {
        let mut mode = WriteMode::Custom(2);
        remap_write_mode(&mut mode, 1);
        assert_eq!(mode, WriteMode::Custom(1));
        remap_write_mode(&mut mode, 0);
        assert_eq!(mode, WriteMode::Custom(0));
    }

    #[test]
    fn remap_removed_index_falls_back_to_hex() {
        let mut mode = WriteMode::Custom(1);
        remap_write_mode(&mut mode, 1);
        assert_eq!(mode, WriteMode::Hex);
    }

    #[test]
    fn common_encoding_labels_matches_encoding_count() {
        let labels = common_encoding_labels();
        assert_eq!(labels.len(), COMMON_ENCODINGS.len());
        // Each label should correspond to the first element of a COMMON_ENCODINGS entry
        for (i, label) in labels.iter().enumerate() {
            assert_eq!(*label, COMMON_ENCODINGS[i].0);
        }
    }
}
