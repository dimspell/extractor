use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::editing::{EditState, InspectorEditState};
use super::provider::{BufferProvider, HexProvider};
use super::selection::Selection;
use super::vanilla_diff::compute_diff;

/// Default cell width — 16 bytes per row matches every other hex editor on
/// the planet and keeps the address column the same width across files.
pub const DEFAULT_BYTES_PER_ROW: u8 = 16;

pub struct HexEditorState {
    pub path: PathBuf,
    pub name: String,
    pub provider: BufferProvider,
    pub bytes_per_row: u8,
    pub selection: Selection,
    pub edit_mode: Option<EditState>,
    pub inspector_edit: Option<InspectorEditState>,
    /// Original bytes used as the diff baseline. Populated either from a
    /// workspace vanilla snapshot or, lacking that, from the on-disk file at
    /// load time. `None` when neither source is available.
    pub vanilla: Option<Vec<u8>>,
    /// Cached set of addresses where `provider != vanilla`. Recomputed on
    /// every write through [`recompute_vanilla_diff`].
    pub vanilla_diff: BTreeSet<u64>,
    /// Last user-facing message produced by an editor action ("Saved …",
    /// "Recording not active", parse errors). Cleared on next save.
    pub status_msg: String,
    pub error: Option<String>,
}

impl HexEditorState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        match std::fs::read(path) {
            Ok(bytes) => {
                // No external vanilla yet → the on-disk content IS the
                // baseline. The save-into-recording path can still upgrade
                // this to a real workspace snapshot later.
                let vanilla = Some(bytes.clone());
                Self {
                    path: path.to_path_buf(),
                    name,
                    provider: BufferProvider::from_bytes(bytes),
                    bytes_per_row: DEFAULT_BYTES_PER_ROW,
                    selection: Selection::default(),
                    edit_mode: None,
                    inspector_edit: None,
                    vanilla,
                    vanilla_diff: BTreeSet::new(),
                    status_msg: String::new(),
                    error: None,
                }
            }
            Err(e) => Self {
                path: path.to_path_buf(),
                name,
                provider: BufferProvider::default(),
                bytes_per_row: DEFAULT_BYTES_PER_ROW,
                selection: Selection::default(),
                edit_mode: None,
                inspector_edit: None,
                vanilla: None,
                vanilla_diff: BTreeSet::new(),
                status_msg: String::new(),
                error: Some(e.to_string()),
            },
        }
    }

    /// Largest valid byte address, or 0 for an empty file.
    pub fn max_addr(&self) -> u64 {
        self.provider.len().saturating_sub(1)
    }

    /// Refresh [`vanilla_diff`] against the current provider contents.
    /// Cheap (linear scan); call after any in-memory write.
    pub fn recompute_vanilla_diff(&mut self) {
        self.vanilla_diff = match &self.vanilla {
            Some(v) => compute_diff(v, self.provider.as_slice()),
            None => BTreeSet::new(),
        };
    }
}
