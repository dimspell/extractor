use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::editing::{EditState, InspectorEditState};
use super::goto::GotoState;
use super::pattern::Pattern;
use super::provider::{BufferProvider, HexProvider};
use super::search::SearchState;
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
    /// Highlighted byte ranges for pattern matching/debugging. In-memory only,
    /// not persisted to disk.
    pub patterns: Vec<Pattern>,
    /// Fast address → pattern_id lookup, rebuilt after every mutation.
    pub pattern_by_addr: BTreeMap<u64, usize>,
    /// Whether the pattern-list panel is visible.
    pub show_pattern_list: bool,
    /// Monotonically increasing id counter for new patterns.
    pub next_pattern_id: usize,
    /// Last address where right-click occurred (for context menu).
    pub context_menu_addr: Option<u64>,
    /// Goto-address dialog state (None when closed).
    pub goto: Option<GotoState>,
    /// Search & replace overlay state.
    pub search: SearchState,
    /// Last user-facing message produced by an editor action ("Saved …",
    /// "Recording not active", parse errors). Cleared on next save.
    /// Toggle: false → hex addresses (default), true → decimal.
    pub show_decimal: bool,
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
                    patterns: Vec::new(),
                    pattern_by_addr: BTreeMap::new(),
                    show_pattern_list: false,
                    next_pattern_id: 0,
                    context_menu_addr: None,
                    goto: None,
                    search: SearchState::new(),
                    show_decimal: false,
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
                patterns: Vec::new(),
                pattern_by_addr: BTreeMap::new(),
                show_pattern_list: false,
                next_pattern_id: 0,
                context_menu_addr: None,
                goto: None,
                search: SearchState::new(),
                show_decimal: false,
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

    /// Add all addresses in [start, end] range as a new pattern.
    /// Returns the pattern id.
    pub fn add_pattern(&mut self, start: u64, end: u64) -> usize {
        let id = self.next_pattern_id;
        self.next_pattern_id += 1;
        let color_idx = (self.patterns.len() % 16) as u8;
        self.patterns.push(Pattern::new(id, start, end, color_idx));
        self.rebuild_pattern_lookup();
        id
    }

    /// Remove a pattern by its id.
    pub fn remove_pattern(&mut self, id: usize) {
        self.patterns.retain(|p| p.id != id);
        self.rebuild_pattern_lookup();
    }

    /// Clear all patterns.
    pub fn clear_patterns(&mut self) {
        self.patterns.clear();
        self.pattern_by_addr.clear();
    }

    /// Rebuild the `pattern_by_addr` lookup from the current `patterns` vec.
    pub fn rebuild_pattern_lookup(&mut self) {
        self.pattern_by_addr.clear();
        for pat in &self.patterns {
            for addr in pat.start..=pat.end {
                self.pattern_by_addr.insert(addr, pat.id);
            }
        }
    }

    /// Return the pattern id for an address if it falls within any pattern.
    pub fn pattern_id_at(&self, addr: u64) -> Option<usize> {
        self.pattern_by_addr.get(&addr).copied()
    }

    /// Return the pattern with the given id, if it exists.
    pub fn pattern_by_id(&self, id: usize) -> Option<&Pattern> {
        self.patterns.iter().find(|p| p.id == id)
    }
}
