use dispel_core::map::fogdata::{FogData, MAX_FACTOR, ROW_LEN, ROWS};
use std::fs::File;
use std::path::{Path, PathBuf};

/// Maximum undo snapshots kept per tab.
const UNDO_CAP: usize = 50;

/// Per-tab editor state for `ExtraInGame/fogdata.dat` — observed
/// map-lighting fade tables (123 levels × 512 brightness factors).
///
/// Undo/redo is snapshot-based (whole `FogData` clones): a snapshot is taken
/// at paint-stroke start and pushed when the stroke ends, or pushed directly
/// before a single-value commit from the inspector.
#[derive(Debug, Clone)]
pub struct FogDataEditorState {
    pub path: PathBuf,
    pub save_path: PathBuf,
    pub name: String,
    /// Parsed fade tables; `None` while unloaded.
    pub fog: Option<FogData>,
    /// Load failure — replaces the whole editor surface. Never used for
    /// transient save errors (those go to the status bar).
    pub error: Option<String>,
    /// Selected light level (1-based, `1..=[ROWS]`).
    pub selected_level: u32,
    /// Selected pixel-pair index (`0..[ROW_LEN]`).
    pub selected_pair: usize,
    pub dirty: bool,
    pub undo_stack: Vec<FogData>,
    pub redo_stack: Vec<FogData>,
    /// Curve snapshot captured when a paint stroke begins; pushed onto the
    /// undo stack when the stroke ends — but only if data actually changed.
    pending_stroke_base: Option<FogData>,
    /// Raw contents of the inspector numeric field.
    pub value_input: String,
    /// Inline validation error for [`Self::value_input`], if invalid.
    pub input_error: Option<String>,
    /// Revert confirmation dialog is open (shown only when dirty).
    pub confirm_revert: bool,
    /// Bumped on every mutating action and on each `Save` dispatch. A
    /// `SaveComplete` carrying a stale generation must not clear `dirty`
    /// (an edit may have landed while the save was in flight).
    pub save_generation: u64,
}

impl Default for FogDataEditorState {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            save_path: PathBuf::new(),
            name: String::new(),
            fog: None,
            error: None,
            selected_level: 1,
            selected_pair: 0,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            pending_stroke_base: None,
            value_input: String::new(),
            input_error: None,
            confirm_revert: false,
            save_generation: 0,
        }
    }
}

impl FogDataEditorState {
    /// Opens and parses a `fogdata.dat` synchronously (the file is 63 KB).
    /// On failure the state is still usable and surfaces [`Self::error`].
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        match File::open(path).and_then(|mut f| FogData::parse(&mut f)) {
            Ok(fog) => {
                let mut state = Self {
                    path: path.to_path_buf(),
                    save_path: path.to_path_buf(),
                    name,
                    fog: Some(fog),
                    ..Default::default()
                };
                state.sync_input_from_selection();
                state
            }
            Err(e) => Self {
                path: path.to_path_buf(),
                save_path: path.to_path_buf(),
                name,
                error: Some(e.to_string()),
                ..Default::default()
            },
        }
    }

    /// Number of light levels in the table (`ROWS`).
    pub fn level_count(&self) -> usize {
        ROWS
    }

    /// The full 512-sample flicker curve of the selected level.
    /// Empty slice when no data is loaded.
    pub fn current_row(&self) -> &[u8] {
        static EMPTY: [u8; 0] = [];
        match &self.fog {
            Some(fog) if self.selected_level >= 1 && (self.selected_level as usize) <= ROWS => {
                fog.row(self.selected_level)
            }
            _ => &EMPTY,
        }
    }

    /// Factor of the currently selected pair, if data is loaded.
    pub fn selected_factor(&self) -> Option<u8> {
        self.fog
            .as_ref()
            .and_then(|f| f.get_factor(self.selected_level, self.selected_pair))
    }

    // ── Editing ───────────────────────────────────────────────────────────

    /// Single funnel into `FogData::set_factor`.
    ///
    /// Kept isolated so the core's error type can change under us
    /// (`io::Error` today, typed `SetFactorError` later) without touching
    /// any call site — everything here speaks plain strings.
    fn apply_factor(&mut self, pair: usize, value: u8) -> Result<(), String> {
        let Some(ref mut fog) = self.fog else {
            return Err("No fog data loaded".to_string());
        };
        // Guard rails before hitting the core so an out-of-range value can
        // never panic or wrap: values >31 wrap silently when consumed.
        if value > MAX_FACTOR {
            return Err(format!("Value must be 0–{MAX_FACTOR}"));
        }
        if pair >= ROW_LEN || self.selected_level == 0 || self.selected_level as usize > ROWS {
            return Err("Pair index out of range".to_string());
        }
        fog.set_factor(self.selected_level, pair, value)
            .map_err(|e| e.to_string())
    }

    /// Capture the pre-stroke curve snapshot (idempotent within a stroke).
    pub fn begin_stroke_if_needed(&mut self) {
        if self.pending_stroke_base.is_none() {
            self.pending_stroke_base = self.fog.clone();
        }
    }

    /// Close the current paint stroke: pushes the stroke-start snapshot onto
    /// the undo stack only when the curve actually changed during the stroke.
    pub fn end_stroke(&mut self) {
        if let Some(base) = self.pending_stroke_base.take()
            && let Some(ref current) = self.fog
            && base != *current
        {
            self.undo_stack.push(base);
            self.redo_stack.clear();
            if self.undo_stack.len() > UNDO_CAP {
                self.undo_stack.remove(0);
            }
        }
    }

    /// Paint one sample during a drag. Returns `true` when the value changed.
    pub fn paint_factor(&mut self, pair: usize, value: u8) -> bool {
        let before = self
            .fog
            .as_ref()
            .and_then(|f| f.get_factor(self.selected_level, pair));
        if self.apply_factor(pair, value).is_err() {
            return false;
        }
        before != Some(value)
    }

    /// Commit a single factor edit with its own undo snapshot.
    /// Returns whether the curve actually changed; state is untouched on
    /// rejection.
    pub fn commit_factor(&mut self, pair: usize, value: u8) -> Result<bool, String> {
        let current = self
            .fog
            .as_ref()
            .and_then(|f| f.get_factor(self.selected_level, pair));
        if current == Some(value) {
            return Ok(false); // No-op edit: no dirty flag, no undo entry.
        }
        let snapshot = self.fog.clone();
        self.apply_factor(pair, value)?;
        if let Some(snapshot) = snapshot {
            self.undo_stack.push(snapshot);
            self.redo_stack.clear();
            if self.undo_stack.len() > UNDO_CAP {
                self.undo_stack.remove(0);
            }
        }
        Ok(true)
    }

    // ── Inspector field ───────────────────────────────────────────────────

    /// Refresh the numeric buffer from the current selection.
    pub fn sync_input_from_selection(&mut self) {
        if let Some(v) = self.selected_factor() {
            self.value_input = v.to_string();
            self.input_error = None;
        }
    }

    /// Live-validate the numeric field: `0..=MAX_FACTOR`, integers only.
    pub fn set_value_input(&mut self, raw: String) {
        self.input_error = validate_value_text(&raw);
        self.value_input = raw;
    }

    /// Commit whatever is in the numeric field to the selected pair.
    /// Returns `Some(value)` on success; on failure the reason is already
    /// stored in [`Self::input_error`].
    pub fn submit_value_input(&mut self) -> Option<u8> {
        if let Some(msg) = validate_value_text(&self.value_input) {
            self.input_error = Some(msg);
            return None;
        }
        let value: u8 = self.value_input.parse().unwrap_or(u8::MAX);
        match self.commit_factor(self.selected_pair, value) {
            Ok(_changed) => {
                self.input_error = None;
                Some(value)
            }
            Err(e) => {
                self.input_error = Some(e);
                None
            }
        }
    }

    // ── Undo / redo ───────────────────────────────────────────────────────

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Restores the previous curve. Returns `true` when something was undone.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_stack.pop() else {
            return false;
        };
        if let Some(current) = self.fog.take() {
            self.redo_stack.push(current);
        }
        self.fog = Some(previous);
        self.dirty = true;
        self.sync_input_from_selection();
        true
    }

    /// Re-applies the last undone edit. Returns `true` when it did anything.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        if let Some(current) = self.fog.take() {
            self.undo_stack.push(current);
        }
        self.fog = Some(next);
        self.dirty = true;
        self.sync_input_from_selection();
        true
    }

    // ── Dirty / revert ────────────────────────────────────────────────────

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Reload the table from disk, discarding all edits and history.
    pub fn reload_from_disk(&mut self) -> Result<(), String> {
        let file = File::open(&self.save_path).map_err(|e| e.to_string())?;
        let mut file = file;
        let fog = FogData::parse(&mut file).map_err(|e| e.to_string())?;
        self.fog = Some(fog);
        self.dirty = false;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.pending_stroke_base = None;
        self.confirm_revert = false;
        self.sync_input_from_selection();
        Ok(())
    }
}

/// Validate raw text as a brightness factor: unsigned integer `0..=31`.
fn validate_value_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    match trimmed.parse::<u8>() {
        Ok(v) if v <= MAX_FACTOR => None,
        Ok(_) => Some(format!("Must be 0–{MAX_FACTOR}")),
        Err(_) if trimmed.is_empty() => Some("Enter a number".to_string()),
        Err(_) => Some("Not a number".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_value_text_accepts_range_and_rejects_rest() {
        assert_eq!(validate_value_text("0"), None);
        assert_eq!(validate_value_text("31"), None);
        assert_eq!(validate_value_text(" 7 "), None);
        assert!(validate_value_text("32").is_some());
        assert!(validate_value_text("-1").is_some());
        assert!(validate_value_text("abc").is_some());
        assert!(validate_value_text("").is_some());
    }
}
