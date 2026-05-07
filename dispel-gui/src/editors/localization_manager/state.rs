use crate::components::loading_state::LoadingState;
use crate::components::textarea::TextAreaContent;
use crate::editors::mod_packager::state::ModMetadata;
use dispel_core::TextEntry;

#[derive(Debug)]
pub struct LocalizationManagerState {
    pub entries: Vec<TextEntry>,
    pub filter_file: Option<String>,
    pub show_untranslated_only: bool,
    pub show_overlong_only: bool,
    pub target_lang: String,
    pub status_msg: String,
    pub loading_state: LoadingState<()>,
    pub mod_metadata: ModMetadata,
    pub selected_idx: Option<usize>,
    pub translation_content: TextAreaContent,
    pub search_query: String,
    pub page: usize,
}

impl Default for LocalizationManagerState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            filter_file: None,
            show_untranslated_only: false,
            show_overlong_only: false,
            target_lang: String::new(),
            status_msg: String::new(),
            loading_state: LoadingState::default(),
            mod_metadata: ModMetadata::default(),
            selected_idx: None,
            translation_content: TextAreaContent::with_text(""),
            search_query: String::new(),
            page: 0,
        }
    }
}

impl LocalizationManagerState {
    pub fn translated_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_translated()).count()
    }

    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    pub fn overlong_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.is_translated() && e.would_truncate())
            .count()
    }

    /// Filtered view of entries based on current filter settings.
    /// Returns (global_index, entry) pairs so the view can dispatch SelectEntry with the correct index.
    pub fn visible_entries(&self) -> Vec<(usize, &TextEntry)> {
        let q = self.search_query.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if e.original.is_empty() {
                    return false;
                }
                let file_ok = self
                    .filter_file
                    .as_deref()
                    .map_or(true, |f| e.file_path.contains(f));
                let translated_ok = !self.show_untranslated_only || !e.is_translated();
                let overlong_ok = !self.show_overlong_only || e.would_truncate();
                let search_ok = q.is_empty()
                    || e.original.to_lowercase().contains(&q)
                    || e.translation.to_lowercase().contains(&q);
                file_ok && translated_ok && overlong_ok && search_ok
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

    /// Path to the session file for the current mod name.
    pub fn session_path(&self, game_path: &str) -> Option<std::path::PathBuf> {
        let name = self.mod_metadata.name.trim();
        if name.is_empty() || game_path.is_empty() {
            return None;
        }
        let mod_name = name.replace(' ', "_");
        Some(
            std::path::PathBuf::from(game_path)
                .join("mods")
                .join(&mod_name)
                .join("session.json"),
        )
    }

    /// Backup directory for the current mod name.
    pub fn backup_dir(&self, game_path: &str) -> Option<std::path::PathBuf> {
        let name = self.mod_metadata.name.trim();
        if name.is_empty() || game_path.is_empty() {
            return None;
        }
        let mod_name = name.replace(' ', "_");
        Some(
            std::path::PathBuf::from(game_path)
                .join("mods")
                .join(&mod_name)
                .join("backup"),
        )
    }

    /// True if a backup exists for the current mod name.
    pub fn backup_exists(&self, game_path: &str) -> bool {
        self.backup_dir(game_path)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Index of the previous untranslated entry before `from`, wrapping around.
    pub fn prev_untranslated(&self, from: usize) -> Option<usize> {
        let n = self.entries.len();
        if n == 0 {
            return None;
        }
        for offset in 1..=n {
            let idx = (from + n - offset) % n;
            if !self.entries[idx].is_translated() {
                return Some(idx);
            }
        }
        None
    }

    /// Index of the next untranslated entry after `from`, wrapping around.
    pub fn next_untranslated(&self, from: usize) -> Option<usize> {
        let n = self.entries.len();
        if n == 0 {
            return None;
        }
        for offset in 1..=n {
            let idx = (from + offset) % n;
            if !self.entries[idx].is_translated() {
                return Some(idx);
            }
        }
        None
    }
}
