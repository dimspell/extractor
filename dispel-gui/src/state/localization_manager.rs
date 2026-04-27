use crate::loading_state::LoadingState;
use crate::state::mod_packager::ModMetadata;
use dispel_core::TextEntry;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct LocalizationManagerState {
    pub entries: Vec<TextEntry>,
    pub filter_file: Option<String>,
    pub show_untranslated_only: bool,
    pub status_msg: String,
    pub loading_state: LoadingState<()>,
    /// Keys of entries whose translation exceeds max_bytes.
    pub truncated_keys: HashSet<(String, usize, String)>,
    pub mod_metadata: ModMetadata,
}

impl LocalizationManagerState {
    pub fn translated_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_translated()).count()
    }

    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Filtered view of entries based on current filter settings.
    pub fn visible_entries(&self) -> Vec<(usize, &TextEntry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let file_ok = self
                    .filter_file
                    .as_deref()
                    .map_or(true, |f| e.file_path.contains(f));
                let translated_ok = !self.show_untranslated_only || !e.is_translated();
                file_ok && translated_ok
            })
            .collect()
    }

    /// Distinct file paths present in entries, sorted.
    pub fn available_files(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .entries
            .iter()
            .map(|e| e.file_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        paths.sort();
        paths
    }

    pub fn is_truncated(&self, file_path: &str, record_id: usize, field_name: &str) -> bool {
        self.truncated_keys
            .contains(&(file_path.to_owned(), record_id, field_name.to_owned()))
    }

    pub fn recompute_truncation(&mut self) {
        self.truncated_keys.clear();
        for e in &self.entries {
            if e.would_truncate() {
                self.truncated_keys
                    .insert((e.file_path.clone(), e.record_id, e.field_name.to_owned()));
            }
        }
    }
}
