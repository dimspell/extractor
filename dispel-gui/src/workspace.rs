use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A workspace tab that can hold any editor or view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTab {
    pub id: usize,
    pub label: String,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub pinned: bool,
}

/// The workspace manages dynamic tabs instead of a fixed Tab enum.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub tabs: Vec<WorkspaceTab>,
    pub active_tab: Option<usize>, // index into tabs
    #[serde(skip)]
    next_id: usize,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            next_id: 0,
        }
    }

    /// Open a new tab, or focus the existing one if already open.
    pub fn open(&mut self, label: String, path: Option<PathBuf>) -> usize {
        // Check if already open
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = Some(idx);
            return idx;
        }
        let id = self.next_id;
        self.next_id += 1;
        let idx = self.tabs.len();
        self.tabs.push(WorkspaceTab {
            id,
            label,
            path,
            modified: false,
            pinned: false,
        });
        self.active_tab = Some(idx);
        idx
    }

    /// Close a tab by index.
    pub fn close(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        let was_active = self.active_tab == Some(idx);
        self.tabs.remove(idx);
        if was_active {
            self.active_tab = if self.tabs.is_empty() {
                None
            } else {
                Some(idx.min(self.tabs.len() - 1))
            };
        } else if let Some(active) = self.active_tab {
            if active > idx {
                self.active_tab = Some(active - 1);
            }
        }
    }

    /// Get the currently active tab.
    pub fn active(&self) -> Option<&WorkspaceTab> {
        self.active_tab.and_then(|idx| self.tabs.get(idx))
    }

    /// Get the currently active tab index.
    pub fn active_index(&self) -> Option<usize> {
        self.active_tab
    }

    /// Mark the active tab as modified.
    pub fn mark_modified(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                tab.modified = true;
            }
        }
    }

    /// Clear the modified flag on the active tab.
    pub fn clear_modified(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                tab.modified = false;
            }
        }
    }
}
