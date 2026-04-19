use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DraftEntry {
    pub file_path: PathBuf,
    pub content: Vec<u8>,
    pub saved_at: SystemTime,
    pub original_mtime: SystemTime,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DraftManager {
    drafts: HashMap<String, DraftEntry>,
    auto_save_enabled: bool,
}

impl DraftManager {
    pub fn new() -> Self {
        Self {
            drafts: HashMap::new(),
            auto_save_enabled: true,
        }
    }

    /// Load draft state from the default disk path, falling back to a fresh instance.
    pub fn load() -> Self {
        let path = Self::persist_path();
        if path.exists() {
            if let Ok(data) = fs::read(&path) {
                if let Ok(manager) = serde_json::from_slice::<DraftManager>(&data) {
                    return manager;
                }
            }
        }
        Self::new()
    }

    /// Persist draft state to disk.
    pub fn save_to_disk(&self) {
        let path = Self::persist_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_vec_pretty(self) {
            let _ = fs::write(&path, data);
        }
    }

    /// Default path: ~/.config/dispel-gui/drafts.json
    fn persist_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("dispel-gui").join("drafts.json")
    }

    pub fn toggle_auto_save(&mut self) {
        self.auto_save_enabled = !self.auto_save_enabled;
    }

    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save_enabled
    }

    pub fn save_draft(&mut self, file_path: &Path, content: &[u8]) {
        let key = file_path.to_string_lossy().to_string();
        let original_mtime = fs::metadata(file_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        self.drafts.insert(
            key,
            DraftEntry {
                file_path: file_path.to_path_buf(),
                content: content.to_vec(),
                saved_at: SystemTime::now(),
                original_mtime,
            },
        );
        self.save_to_disk();
    }

    pub fn has_draft(&self, file_path: &Path) -> bool {
        let key = file_path.to_string_lossy().to_string();
        self.drafts.contains_key(&key)
    }

    pub fn get_draft(&self, file_path: &Path) -> Option<&DraftEntry> {
        let key = file_path.to_string_lossy().to_string();
        self.drafts.get(&key)
    }

    pub fn clear_draft(&mut self, file_path: &Path) {
        let key = file_path.to_string_lossy().to_string();
        self.drafts.remove(&key);
    }

    pub fn check_conflicts(&self) -> Vec<ConflictInfo> {
        let mut conflicts = Vec::new();

        for (key, draft) in &self.drafts {
            if let Ok(metadata) = fs::metadata(&draft.file_path) {
                if let Ok(current_mtime) = metadata.modified() {
                    if current_mtime > draft.original_mtime {
                        conflicts.push(ConflictInfo {
                            file_path: draft.file_path.clone(),
                            draft_saved_at: draft.saved_at,
                            file_modified_at: current_mtime,
                        });
                    }
                }
            }
            let _ = key;
        }

        conflicts
    }

    pub fn apply_draft(&self, file_path: &Path) -> Result<(), String> {
        if let Some(draft) = self.get_draft(file_path) {
            fs::write(&draft.file_path, &draft.content)
                .map_err(|e| format!("Failed to write draft: {}", e))?;
            Ok(())
        } else {
            Err("No draft found for this file".to_string())
        }
    }

    pub fn discard_draft(&mut self, file_path: &Path) {
        self.clear_draft(file_path);
        self.save_to_disk();
    }

    pub fn draft_count(&self) -> usize {
        self.drafts.len()
    }

    pub fn pending_drafts(&self) -> Vec<&DraftEntry> {
        self.drafts.values().collect()
    }
}

#[derive(Clone, Debug)]
pub struct ConflictInfo {
    pub file_path: PathBuf,
    pub draft_saved_at: SystemTime,
    pub file_modified_at: SystemTime,
}

impl ConflictInfo {
    pub fn display_text(&self) -> String {
        let filename = self
            .file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.file_path.to_string_lossy().to_string());
        format!(
            "{} was modified externally after your draft was saved",
            filename
        )
    }
}
