//! Localization support: extract translatable text from game records, export/import
//! CSV and PO files, and apply translations back to game structs.

use std::collections::HashMap;

// `TextEncoding` (accepted spellings, labels and `encoding_rs` mapping) lives
// in the shared `dispel-encodings` registry so the derive macros and the
// runtime helpers cannot drift apart.
pub use dispel_encodings::TextEncoding;

// ─── Core types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextEntry {
    pub file_path: String,
    pub record_id: usize,
    pub field_name: &'static str,
    pub original: String,
    pub translation: String,
    pub encoding: TextEncoding,
    pub max_bytes: usize,
}

impl TextEntry {
    /// The canonical typed address of this translatable cell. Lets a
    /// localization entry and the equivalent modding `FieldDelta` be compared
    /// as one identity instead of two parallel `(file, record, field)` tuples.
    pub fn field_address(&self) -> crate::field_identity::FieldAddress {
        crate::field_identity::FieldAddress::flat(
            self.file_path.clone(),
            self.record_id as u32,
            self.field_name,
        )
    }

    /// True when a non-empty, changed translation exists.
    pub fn is_translated(&self) -> bool {
        !self.translation.is_empty() && self.translation != self.original
    }

    /// Byte length of `translation` when encoded with the target encoding.
    pub fn encoded_translation_len(&self) -> usize {
        encoded_len(&self.translation, &self.encoding)
    }

    /// Whether the translation would exceed `max_bytes` after encoding.
    pub fn would_truncate(&self) -> bool {
        self.encoded_translation_len() > self.max_bytes
    }
}

#[derive(Debug, Clone)]
pub enum TruncationStatus {
    Ok,
    Truncated { original_bytes: usize },
}

// ─── Localizable trait ───────────────────────────────────────────────────────

/// Implemented (via `#[derive(Localizable)]`) for structs whose fields carry
/// `#[translatable(encoding = "...", max_bytes = N)]` attributes.
pub trait Localizable {
    fn extract_texts(&self, record_id: usize, file_path: &str) -> Vec<TextEntry>;
    fn apply_texts(&mut self, entries: &[TextEntry]) -> Vec<TruncationStatus>;
}

// ─── Encoding helpers ────────────────────────────────────────────────────────

fn encoding_rs_for(enc: &TextEncoding) -> &'static encoding_rs::Encoding {
    enc.encoding_rs()
}

fn encoded_len(s: &str, enc: &TextEncoding) -> usize {
    let (bytes, _, _) = encoding_rs_for(enc).encode(s);
    bytes.len()
}

/// Truncate `s` so it fits within `max_bytes` when re-encoded, working character by character.
/// Returns `(truncated_string, was_truncated)`.
pub fn truncate_to_fit(s: &str, enc: &TextEncoding, max_bytes: usize) -> (String, bool) {
    let encoder = encoding_rs_for(enc);
    let (full, _, _) = encoder.encode(s);
    if full.len() <= max_bytes {
        return (s.to_owned(), false);
    }
    let mut byte_count = 0usize;
    let mut result = String::new();
    for ch in s.chars() {
        let ch_str = ch.to_string();
        let (ch_bytes, _, _) = encoder.encode(&ch_str);
        if byte_count + ch_bytes.len() > max_bytes {
            break;
        }
        byte_count += ch_bytes.len();
        result.push(ch);
    }
    (result, true)
}

// ─── CSV export / import ─────────────────────────────────────────────────────

/// Export entries to UTF-8 CSV with headers.
/// Columns: file_path, record_id, field_name, original, translation, encoding, max_bytes
pub fn export_csv(entries: &[TextEntry]) -> Result<String, csv::Error> {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record([
        "file_path",
        "record_id",
        "field_name",
        "original",
        "translation",
        "encoding",
        "max_bytes",
    ])?;
    for e in entries {
        wtr.write_record([
            e.file_path.as_str(),
            &e.record_id.to_string(),
            e.field_name,
            e.original.as_str(),
            e.translation.as_str(),
            e.encoding.label(),
            &e.max_bytes.to_string(),
        ])?;
    }
    wtr.flush()?;
    let bytes = wtr
        .into_inner()
        .map_err(|e| csv::Error::from(e.into_error()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Outcome of importing translations from CSV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvApplyResult {
    /// Data rows whose translation was applied to a matching entry.
    pub updated: usize,
    /// Data rows that were not applied: the key was invalid (missing file,
    /// field or record id) or matched no known entry.
    pub skipped: usize,
}

/// Import translations from CSV; matches rows by (file_path, record_id, field_name).
/// Only updates the `translation` field of matching entries. Rows with an
/// unparseable record id or a missing file/field are skipped rather than being
/// coerced onto record 0; both applied and skipped counts are reported.
pub fn import_csv(csv: &str, entries: &mut [TextEntry]) -> Result<CsvApplyResult, csv::Error> {
    let mut rdr = csv::Reader::from_reader(csv.as_bytes());
    // Build a lookup: (file_path, record_id, field_name) → index in entries
    let mut index: HashMap<(String, usize, String), usize> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        index.insert(
            (e.file_path.clone(), e.record_id, e.field_name.to_owned()),
            i,
        );
    }
    let mut updated = 0;
    let mut skipped = 0;
    for record in rdr.records() {
        let record = record?;
        let Some(file_path) = record.get(0).filter(|s| !s.is_empty()) else {
            skipped += 1;
            continue;
        };
        let Some(field_name) = record.get(2).filter(|s| !s.is_empty()) else {
            skipped += 1;
            continue;
        };
        let Ok(record_id) = record.get(1).unwrap_or("").parse::<usize>() else {
            skipped += 1;
            continue;
        };
        let translation = record.get(4).unwrap_or("").to_owned();
        if let Some(&idx) = index.get(&(file_path.to_owned(), record_id, field_name.to_owned())) {
            entries[idx].translation = translation;
            updated += 1;
        } else {
            skipped += 1;
        }
    }
    Ok(CsvApplyResult { updated, skipped })
}

// ─── PO export / import ──────────────────────────────────────────────────────

/// Export entries to a GNU gettext PO file.
/// `msgid` = original, `msgstr` = translation (empty string if untranslated).
pub fn export_po(entries: &[TextEntry], source_lang: &str, target_lang: &str) -> String {
    let mut out = String::new();
    out.push_str("# Generated by dispel-extractor Localization Manager\n");
    out.push_str(&format!("# Source language: {source_lang}\n"));
    out.push_str(&format!("# Target language: {target_lang}\n"));
    out.push_str("#\n");
    out.push_str("msgid \"\"\n");
    out.push_str("msgstr \"\"\n");
    out.push_str(&format!("\"Language: {target_lang}\\n\"\n"));
    out.push_str("\"Content-Type: text/plain; charset=UTF-8\\n\"\n");
    out.push_str("\"Content-Transfer-Encoding: 8bit\\n\"\n\n");

    for e in entries {
        out.push_str(&format!(
            "#. field: {}, max_bytes: {}, encoding: {}\n",
            e.field_name,
            e.max_bytes,
            e.encoding.label()
        ));
        out.push_str(&format!("#: {}:{}\n", e.file_path, e.record_id));
        out.push_str(&format!("msgid {}\n", po_quote(&e.original)));
        out.push_str(&format!("msgstr {}\n\n", po_quote(&e.translation)));
    }
    out
}

/// Import translations from a PO file; matches by `msgid` value.
/// When multiple entries share the same `msgid`, all are updated.
/// Returns the number of `msgstr` values applied.
pub fn import_po(po: &str, entries: &mut [TextEntry]) -> usize {
    // Build a lookup: original → list of indices
    let mut by_original: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        by_original.entry(e.original.clone()).or_default().push(i);
    }

    // Apply a completed msgid/msgstr pair to every entry sharing the original.
    // Called when a new msgid begins, a blank line ends the entry, and at EOF.
    let flush_entry =
        |id: &Option<String>, msgstr: &str, entries: &mut [TextEntry], updated: &mut usize| {
            let Some(id) = id else {
                return;
            };
            if msgstr.is_empty() {
                return;
            }
            let Some(idxs) = by_original.get(id) else {
                return;
            };
            for &i in idxs {
                entries[i].translation = msgstr.to_owned();
                *updated += 1;
            }
        };

    let mut updated = 0;
    let mut current_msgid: Option<String> = None;
    let mut in_msgstr = false;
    let mut msgstr_buf = String::new();

    for line in po.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("msgid ") {
            if in_msgstr {
                flush_entry(&current_msgid, &msgstr_buf, entries, &mut updated);
            }
            in_msgstr = false;
            msgstr_buf.clear();
            let value = po_unquote(stripped);
            current_msgid = if value.is_empty() { None } else { Some(value) };
        } else if let Some(stripped) = line.strip_prefix("msgstr ") {
            in_msgstr = true;
            msgstr_buf = po_unquote(stripped);
        } else if in_msgstr && line.starts_with('"') {
            msgstr_buf.push_str(&po_unquote(line));
        } else if line.is_empty() && in_msgstr {
            // end of entry
            flush_entry(&current_msgid, &msgstr_buf, entries, &mut updated);
            in_msgstr = false;
            current_msgid = None;
            msgstr_buf.clear();
        }
    }
    // flush last entry
    flush_entry(&current_msgid, &msgstr_buf, entries, &mut updated);
    updated
}

fn po_quote(s: &str) -> String {
    if s.is_empty() {
        return "\"\"".to_owned();
    }
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{}\"", escaped)
}

fn po_unquote(s: &str) -> String {
    let s = s.trim();
    if !(s.starts_with('"') && s.ends_with('"') && s.len() >= 2) {
        return s.to_owned();
    }
    // Consume escapes left-to-right so a literal backslash before n/r/quote
    // is not mistaken for the start of an escape. Unknown escapes are kept.
    let inner = &s[1..s.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<TextEntry> {
        vec![
            TextEntry {
                file_path: "CharacterInGame/STORE.DB".to_owned(),
                record_id: 0,
                field_name: "store_name",
                original: "Weapon Shop".to_owned(),
                translation: String::new(),
                encoding: TextEncoding::Windows1250,
                max_bytes: 32,
            },
            TextEntry {
                file_path: "CharacterInGame/STORE.DB".to_owned(),
                record_id: 0,
                field_name: "invitation",
                original: "Welcome, traveller.".to_owned(),
                translation: "Witaj, wędrowcze.".to_owned(),
                encoding: TextEncoding::Windows1250,
                max_bytes: 512,
            },
        ]
    }

    fn text_entry(original: &str) -> TextEntry {
        TextEntry {
            file_path: "test.db".to_owned(),
            record_id: 1,
            field_name: "name",
            original: original.to_owned(),
            translation: String::new(),
            encoding: TextEncoding::Windows1250,
            max_bytes: 64,
        }
    }

    #[test]
    fn csv_round_trip() {
        let entries = sample_entries();
        let csv = export_csv(&entries).unwrap();
        let mut entries2 = sample_entries();
        let result = import_csv(&csv, &mut entries2).unwrap();
        assert_eq!(
            result,
            CsvApplyResult {
                updated: 2,
                skipped: 0
            }
        );
        assert_eq!(entries2[1].translation, "Witaj, wędrowcze.");
    }

    #[test]
    fn csv_import_skips_invalid_keys_without_touching_entry_zero() {
        let mut entries = sample_entries();
        entries[0].translation = "sentinel".to_owned();
        let csv = "file_path,record_id,field_name,original,translation,encoding,max_bytes\n\
                   CharacterInGame/STORE.DB,,store_name,Weapon Shop,CORRUPT,WINDOWS-1250,32\n\
                   CharacterInGame/STORE.DB,garbage,invitation,Welcome,BAD,WINDOWS-1250,512\n\
                   CharacterInGame/STORE.DB,0,,Weapon Shop,NOFIELD,WINDOWS-1250,32\n";
        let result = import_csv(csv, &mut entries).unwrap();
        assert_eq!(
            result,
            CsvApplyResult {
                updated: 0,
                skipped: 3
            }
        );
        assert_eq!(entries[0].translation, "sentinel");
        assert_eq!(entries[1].translation, "Witaj, wędrowcze.");
    }

    #[test]
    fn po_round_trip() {
        let mut entries = sample_entries();
        entries[0].translation = "Sklep z bronią".to_owned();
        let po = export_po(&entries, "ko", "pl");
        let mut entries2 = sample_entries();
        let updated = import_po(&po, &mut entries2);
        assert_eq!(updated, 2);
        assert_eq!(entries2[0].translation, "Sklep z bronią");
        assert_eq!(entries2[1].translation, "Witaj, wędrowcze.");
    }

    #[test]
    fn po_unquote_decodes_escapes_left_to_right() {
        assert_eq!(po_unquote("\"C:\\\\new\""), "C:\\new");
        assert_eq!(po_unquote("\"a\\\\\\\"b\""), "a\\\"b");
        assert_eq!(po_unquote("\"x\\\\\\\\y\""), "x\\\\y");
        assert_eq!(po_unquote("\"a\\\\nb\""), "a\\nb");
        assert_eq!(po_unquote("\"\\\""), "\\");
        assert_eq!(po_unquote("\"a\\nb\""), "a\nb");
        assert_eq!(po_unquote("\"a\\rb\""), "a\rb");
        assert_eq!(po_unquote("\"a\\qb\""), "a\\qb");
    }

    #[test]
    fn po_quote_output_round_trips_through_unquote() {
        for original in [
            "C:\\new",
            "a\"b",
            "x\\y",
            "line1\nline2",
            "a\r\nb",
            "\\",
            "\\n",
        ] {
            assert_eq!(
                po_unquote(&po_quote(original)),
                original,
                "failed for {original:?}"
            );
        }
    }

    #[test]
    fn po_import_flushes_at_msgid_blank_line_and_eof() {
        let po = concat!(
            "msgid \"one\"\n",
            "msgstr \"Uno\"\n",
            "msgid \"two\"\n",
            "msgstr \"Dos\"\n",
            "\n",
            "msgid \"three\"\n",
            "msgstr \"Tres\"\n",
        );
        let mut entries = vec![text_entry("one"), text_entry("two"), text_entry("three")];
        let updated = import_po(po, &mut entries);
        assert_eq!(updated, 3);
        assert_eq!(entries[0].translation, "Uno");
        assert_eq!(entries[1].translation, "Dos");
        assert_eq!(entries[2].translation, "Tres");
    }

    #[test]
    fn truncate_single_byte() {
        let (s, truncated) = truncate_to_fit("Hello world", &TextEncoding::Windows1250, 5);
        assert!(truncated);
        assert_eq!(s, "Hello");
    }

    #[test]
    fn truncate_no_op_when_fits() {
        let (s, truncated) = truncate_to_fit("Hi", &TextEncoding::Windows1250, 10);
        assert!(!truncated);
        assert_eq!(s, "Hi");
    }

    #[test]
    fn is_translated() {
        let mut e = sample_entries().remove(0);
        assert!(!e.is_translated());
        e.translation = "X".to_owned();
        assert!(e.is_translated());
        e.translation = e.original.clone();
        assert!(!e.is_translated());
    }

    #[test]
    fn every_accepted_encoding_spelling_resolves_identically() {
        // The derive macros use the very same `dispel-encodings` registry, so
        // checking every spelling here guarantees attribute strings and the
        // runtime `TextEncoding` cannot disagree.
        for encoding in TextEncoding::ALL {
            for spelling in encoding.spellings() {
                assert_eq!(
                    TextEncoding::from_spelling(spelling),
                    Some(encoding),
                    "spelling {spelling:?} must resolve to {encoding:?}"
                );
                assert_eq!(
                    encoding_rs_for(&TextEncoding::from_spelling(spelling).unwrap()).name(),
                    encoding.encoding_rs().name(),
                );
            }
        }
    }

    /// A real derive-based record, exercising both a required `String` field
    /// and an `Option<String>` field with non-UTF-8 encodings.
    #[derive(Default, dispel_macros::Localizable)]
    struct LocalizedStore {
        id: u32,
        #[translatable(encoding = "EUC-KR", max_bytes = 32)]
        name: String,
        #[translatable(encoding = "WINDOWS-1250", max_bytes = 16)]
        note: Option<String>,
    }

    #[test]
    fn localizable_struct_round_trips_non_utf8_encoding() {
        let mut record = LocalizedStore {
            id: 7,
            name: "상점".to_owned(),
            note: Some("Witaj".to_owned()),
        };
        assert_eq!(record.id, 7);
        let mut entries = record.extract_texts(7, "CharacterInGame/STORE.DB");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].field_name, "name");
        assert_eq!(entries[0].encoding, TextEncoding::EucKr);
        assert_eq!(entries[1].field_name, "note");
        assert_eq!(entries[1].encoding, TextEncoding::Windows1250);

        entries[0].translation = "무기 상점".to_owned();
        entries[1].translation = "Sklep z bronią".to_owned();
        let statuses = record.apply_texts(&entries);
        assert!(statuses.iter().all(|s| matches!(s, TruncationStatus::Ok)));
        assert_eq!(record.name, "무기 상점");
        assert_eq!(record.note.as_deref(), Some("Sklep z bronią"));

        // Verify the stored text really round-trips through the declared
        // non-UTF-8 codecs and stays within the declared byte budgets.
        let (name_bytes, _, name_errors) = encoding_rs::EUC_KR.encode(&record.name);
        assert!(!name_errors);
        assert!(name_bytes.len() <= 32);
        let (note_bytes, _, note_errors) =
            encoding_rs::WINDOWS_1250.encode(record.note.as_deref().unwrap());
        assert!(!note_errors);
        assert!(note_bytes.len() <= 16);
        assert_eq!(encoding_rs::EUC_KR.decode(&name_bytes).0, "무기 상점");
    }
}
