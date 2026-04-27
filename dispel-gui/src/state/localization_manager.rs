use crate::loading_state::LoadingState;
use crate::state::mod_packager::ModMetadata;
use dispel_core::TextEntry;

#[derive(Debug, Default)]
pub struct LocalizationManagerState {
    pub entries: Vec<TextEntry>,
    pub filter_file: Option<String>,
    pub show_untranslated_only: bool,
    pub show_overlong_only: bool,
    pub target_lang: String,
    pub status_msg: String,
    pub loading_state: LoadingState<()>,
    pub mod_metadata: ModMetadata,
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
                let overlong_ok = !self.show_overlong_only || e.would_truncate();
                file_ok && translated_ok && overlong_ok
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
}
