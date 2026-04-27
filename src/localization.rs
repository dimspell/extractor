//! Localization support: extract translatable text from game records, export/import
//! CSV and PO files, and apply translations back to game structs.

use std::collections::HashMap;

// ─── Core types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextEncoding {
    Windows1250,
    EucKr,
    Utf8,
}

impl TextEncoding {
    pub fn label(&self) -> &'static str {
        match self {
            TextEncoding::Windows1250 => "WINDOWS-1250",
            TextEncoding::EucKr => "EUC-KR",
            TextEncoding::Utf8 => "UTF-8",
        }
    }
}

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
    match enc {
        TextEncoding::Windows1250 => encoding_rs::WINDOWS_1250,
        TextEncoding::EucKr => encoding_rs::EUC_KR,
        TextEncoding::Utf8 => encoding_rs::UTF_8,
    }
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
    wtr.write_record(["file_path", "record_id", "field_name", "original", "translation", "encoding", "max_bytes"])?;
    for e in entries {
        wtr.write_record(&[
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
    let bytes = wtr.into_inner().map_err(|e| csv::Error::from(e.into_error()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Import translations from CSV; matches rows by (file_path, record_id, field_name).
/// Only updates the `translation` field of matching entries; returns error count.
pub fn import_csv(csv: &str, entries: &mut Vec<TextEntry>) -> Result<usize, csv::Error> {
    let mut rdr = csv::Reader::from_reader(csv.as_bytes());
    // Build a lookup: (file_path, record_id, field_name) → index in entries
    let mut index: HashMap<(String, usize, String), usize> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        index.insert((e.file_path.clone(), e.record_id, e.field_name.to_owned()), i);
    }
    let mut updated = 0;
    for record in rdr.records() {
        let record = record?;
        let file_path = record.get(0).unwrap_or("").to_owned();
        let record_id: usize = record.get(1).unwrap_or("0").parse().unwrap_or(0);
        let field_name = record.get(2).unwrap_or("").to_owned();
        let translation = record.get(4).unwrap_or("").to_owned();
        if let Some(&idx) = index.get(&(file_path, record_id, field_name)) {
            entries[idx].translation = translation;
            updated += 1;
        }
    }
    Ok(updated)
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
pub fn import_po(po: &str, entries: &mut Vec<TextEntry>) -> usize {
    // Build a lookup: original → list of indices
    let mut by_original: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        by_original.entry(e.original.clone()).or_default().push(i);
    }

    let mut updated = 0;
    let mut current_msgid: Option<String> = None;
    let mut in_msgstr = false;
    let mut msgstr_buf = String::new();

    for line in po.lines() {
        let line = line.trim();
        if line.starts_with("msgid ") {
            if in_msgstr {
                // flush previous pair
                if let Some(ref id) = current_msgid {
                    if !msgstr_buf.is_empty() {
                        if let Some(idxs) = by_original.get(id) {
                            for &i in idxs {
                                entries[i].translation = msgstr_buf.clone();
                                updated += 1;
                            }
                        }
                    }
                }
            }
            in_msgstr = false;
            msgstr_buf.clear();
            let value = po_unquote(&line["msgid ".len()..]);
            current_msgid = if value.is_empty() { None } else { Some(value) };
        } else if line.starts_with("msgstr ") {
            in_msgstr = true;
            msgstr_buf = po_unquote(&line["msgstr ".len()..]);
        } else if in_msgstr && line.starts_with('"') {
            msgstr_buf.push_str(&po_unquote(line));
        } else if line.is_empty() && in_msgstr {
            // end of entry
            if let Some(ref id) = current_msgid {
                if !msgstr_buf.is_empty() {
                    if let Some(idxs) = by_original.get(id) {
                        for &i in idxs {
                            entries[i].translation = msgstr_buf.clone();
                            updated += 1;
                        }
                    }
                }
            }
            in_msgstr = false;
            current_msgid = None;
            msgstr_buf.clear();
        }
    }
    // flush last entry
    if in_msgstr {
        if let Some(ref id) = current_msgid {
            if !msgstr_buf.is_empty() {
                if let Some(idxs) = by_original.get(id) {
                    for &i in idxs {
                        entries[i].translation = msgstr_buf.clone();
                        updated += 1;
                    }
                }
            }
        }
    }
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
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        let inner = &s[1..s.len() - 1];
        inner
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        s.to_owned()
    }
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

    #[test]
    fn csv_round_trip() {
        let entries = sample_entries();
        let csv = export_csv(&entries).unwrap();
        let mut entries2 = sample_entries();
        let updated = import_csv(&csv, &mut entries2).unwrap();
        assert_eq!(updated, 2);
        assert_eq!(entries2[1].translation, "Witaj, wędrowcze.");
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
}
