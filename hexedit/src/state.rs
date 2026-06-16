use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use gui_widgets::components::paragraph_cache::ParagraphCache;
use iced::widget::pane_grid;

use super::domain::byte_stats::{ByteStatistics, RowEntropyCache};
use super::domain::export_config::ExportConfig;
use super::domain::fill_dialog::FillDialog;
use super::domain::panel::{default_pane_grid, HexPanel};
use super::domain::write_mode::{EncodingEntry, WriteMode};
use super::editing::{EditState, InspectorEditState};
use super::goto::GotoState;
use super::lua_engine::LuaScriptEngine;
use super::pattern::{Pattern, RepeatPatternDialog, RepeatedPatternGroup};
use super::provider::{BufferProvider, HexProvider};
use super::search::SearchState;
use super::selection::Selection;
use super::ui::coloring::ColorScheme;
use super::vanilla_diff::compute_diff;

/// Default cell width — 16 bytes per row matches every other hex editor on
/// the planet and keeps the address column the same width across files.
pub const DEFAULT_BYTES_PER_ROW: u8 = 16;

pub struct HexEditorState {
    pub path: PathBuf,
    pub name: String,
    /// Halloy-style pane grid: movable, splittable, resizable panels.
    pub panes: pane_grid::State<HexPanel>,
    /// Which pane currently has keyboard focus in the grid.
    pub pane_focus: pane_grid::Pane,
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
    pub pattern_by_addr: BTreeMap<u64, (usize, u8)>,
    /// Whether the pattern-list panel is visible.
    pub show_pattern_list: bool,
    /// Monotonically increasing id counter for new patterns.
    pub next_pattern_id: usize,
    /// Metadata for repeated-pattern groups (label, colour).
    pub groups: Vec<RepeatedPatternGroup>,
    /// Monotonically increasing id counter for new groups.
    pub next_group_id: usize,
    /// Set of group ids whose accordion section is collapsed.
    pub collapsed_groups: BTreeSet<usize>,
    /// Precomputed map: row-start-address → list of `(pattern_id, text)`
    /// annotation segments for the hex matrix annotation column. Rebuilt
    /// after every pattern mutation.
    pub row_annotations: BTreeMap<u64, Vec<(usize, String)>>,
    /// Pattern ids whose span contains the current cursor address. The matrix
    /// uses this to decide which annotation segments to highlight.
    pub active_patterns: BTreeSet<usize>,
    /// Which group is currently being renamed (inline edit in pattern list).
    pub renaming_group: Option<usize>,
    /// Draft text for the rename text input.
    pub renaming_group_draft: String,
    /// Last address where right-click occurred (for context menu).
    pub context_menu_addr: Option<u64>,
    /// Goto-address dialog state (None when closed).
    pub goto: Option<GotoState>,
    /// Export-as-text config modal state (None when closed).
    pub export_config: Option<ExportConfig>,
    /// Fill-selection dialog state (None when closed).
    pub fill_dialog: Option<FillDialog>,
    /// Search & replace overlay state.
    pub search: SearchState,
    /// Last user-facing message produced by an editor action ("Saved …",
    /// "Recording not active", parse errors). Cleared on next save.
    /// Toggle: false → hex addresses (default), true → decimal.
    pub show_decimal: bool,
    pub status_msg: String,
    pub error: Option<String>,
    /// Dialog state for creating a repeated (zebra-striped) pattern.
    /// `None` when the dialog is closed.
    pub repeat_pattern: Option<RepeatPatternDialog>,
    /// Shared paragraph cache shared across frames so shaped glyphs survive
    /// between render cycles (cheaply cloned into the widget each frame).
    pub cache: ParagraphCache,
    /// Which byte-colouring scheme the hex matrix should use.
    pub color_scheme: ColorScheme,
    /// When true, `0x00` bytes are drawn with a dim colour regardless of the
    /// active scheme (Monochrome included).
    pub dim_nulls: bool,
    /// Whether the settings modal is currently open.
    pub settings_open: bool,
    /// Lua scripting engine for custom inspector decoders.
    pub lua_engine: LuaScriptEngine,

    // ── Write mode / text encoding ──────────────────────────────────────
    /// Active write mode for keyboard input.
    /// - Hex   → type two hex digits per byte (existing behaviour).
    /// - Text  → type characters that get encoded into bytes.
    pub write_mode: WriteMode,
    /// User-defined custom text encodings (populated from config or modal).
    pub custom_encodings: Vec<EncodingEntry>,
    /// Whether the "encoding settings" modal is open.
    pub encoding_settings_open: bool,
    /// Index of the encoding the user is hovering in the "add encoding"
    /// pick list inside the encoding-settings modal.
    pub encoding_settings_selection: Option<usize>,

    // ── Byte statistics / entropy panel ────────────────────────────────
    /// Whether the byte statistics panel is visible.
    pub show_stats: bool,
    /// Cached file-level byte statistics (computed on demand via `AnalyzeFile`).
    pub file_stats: Option<ByteStatistics>,
    /// Cached selection-level byte statistics (computed on demand via `AnalyzeSelection`).
    pub selection_stats: Option<ByteStatistics>,
    /// Pre-computed per-row entropy values for colour bands.
    pub row_entropies: Option<RowEntropyCache>,
}

impl HexEditorState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (provider, vanilla, error) = match std::fs::read(path) {
            Ok(bytes) => (BufferProvider::from_bytes(bytes.clone()), Some(bytes), None),
            Err(e) => (BufferProvider::default(), None, Some(e.to_string())),
        };

        let unsafe_mode = std::env::var("HEXEDIT_LUA_UNSAFE").as_deref() == Ok("1");
        let lua_engine = LuaScriptEngine::new(unsafe_mode).unwrap_or_default();

        let panes = default_pane_grid();
        let pane_focus = *panes
            .iter()
            .next()
            .map(|(id, _)| id)
            .expect("default_pane_grid always has at least one pane");

        Self {
            path: path.to_path_buf(),
            name,
            panes,
            pane_focus,
            provider,
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
            groups: Vec::new(),
            next_group_id: 0,
            collapsed_groups: BTreeSet::new(),
            row_annotations: BTreeMap::new(),
            active_patterns: BTreeSet::new(),
            renaming_group: None,
            renaming_group_draft: String::new(),
            context_menu_addr: None,
            goto: None,
            export_config: None,
            fill_dialog: None,
            search: SearchState::new(),
            show_decimal: false,
            status_msg: String::new(),
            error,
            repeat_pattern: None,
            color_scheme: ColorScheme::Monochrome,
            dim_nulls: true,
            settings_open: false,
            cache: ParagraphCache::default(),
            lua_engine,
            write_mode: WriteMode::Hex,
            custom_encodings: Vec::new(),
            encoding_settings_open: false,
            encoding_settings_selection: None,

            show_stats: false,
            file_stats: None,
            selection_stats: None,
            row_entropies: None,
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
        self.recompute_row_annotations();
        id
    }

    /// Remove a pattern by its id. Also removes any group that ends up with
    /// zero patterns (orphan cleanup), and cleans up stale collapsed_groups.
    pub fn remove_pattern(&mut self, id: usize) {
        self.patterns.retain(|p| p.id != id);
        // Clean up orphan groups — groups with no patterns left.
        self.groups
            .retain(|g| self.patterns.iter().any(|p| p.group_id == Some(g.id)));
        // Also clean up stale collapsed_groups entries (orphaned group ids
        // that lingered after the group was removed above).
        self.collapsed_groups
            .retain(|gid| self.groups.iter().any(|g| g.id == *gid));
        self.rebuild_pattern_lookup();
        self.recompute_row_annotations();
    }

    /// Clear all patterns and pattern groups.
    pub fn clear_patterns(&mut self) {
        self.patterns.clear();
        self.pattern_by_addr.clear();
        self.groups.clear();
        self.collapsed_groups.clear();
        self.row_annotations.clear();
        self.active_patterns.clear();
        self.renaming_group = None;
    }

    /// Rebuild the `pattern_by_addr` lookup from the current `patterns` vec.
    pub fn rebuild_pattern_lookup(&mut self) {
        self.pattern_by_addr.clear();
        for pat in &self.patterns {
            for addr in pat.start..=pat.end {
                self.pattern_by_addr.insert(addr, (pat.id, pat.color_idx));
            }
        }
    }

    /// Return the pattern id for an address if it falls within any pattern.
    pub fn pattern_id_at(&self, addr: u64) -> Option<usize> {
        self.pattern_by_addr.get(&addr).map(|(id, _)| *id)
    }

    /// Return the pattern with the given id, if it exists.
    pub fn pattern_by_id(&self, id: usize) -> Option<&Pattern> {
        self.patterns.iter().find(|p| p.id == id)
    }

    /// Rebuild the `row_annotations` map from the current pattern list.
    /// Called after every pattern mutation so the hex matrix can render
    /// annotations to the right of the ASCII column.
    pub fn recompute_row_annotations(&mut self) {
        self.row_annotations.clear();
        let bpr = self.bytes_per_row.max(1) as u64;
        for pat in &self.patterns {
            let Some(ref ann) = pat.annotation else {
                continue;
            };
            if ann.is_empty() {
                continue;
            }
            let first_row = pat.start / bpr;
            let last_row = pat.end / bpr;
            for row in first_row..=last_row {
                let row_start = row * bpr;
                self.row_annotations
                    .entry(row_start)
                    .or_default()
                    .push((pat.id, ann.clone()));
            }
        }
        self.refresh_active_patterns();
    }

    /// Rebuild `active_patterns` — the set of pattern ids whose span contains
    /// the current cursor address. Called automatically from
    /// `recompute_row_annotations()` and should also be called whenever the
    /// cursor moves (select, navigate, etc.).
    ///
    /// Used by the hex matrix to highlight annotation segments and by the
    /// pattern list to highlight the row whose span the cursor is over.
    pub fn refresh_active_patterns(&mut self) {
        self.active_patterns.clear();
        let cursor = self.selection.cursor;
        for pat in &self.patterns {
            if cursor >= pat.start && cursor <= pat.end {
                self.active_patterns.insert(pat.id);
            }
        }
    }

    /// Clear cached byte statistics when file content or row width changes.
    /// Call after every write mutation and after `bytes_per_row` changes.
    pub fn invalidate_stats(&mut self) {
        self.file_stats = None;
        self.selection_stats = None;
        self.row_entropies = None;
    }

    /// Load all `.lua` scripts from a directory into the Lua engine.
    /// Errors are collected and returned; successfully loaded decoders are
    /// available via `lua_engine.entries()`.
    pub fn load_lua_scripts(&mut self, dir: &Path) -> Vec<String> {
        let mut errors = Vec::new();
        if !dir.is_dir() {
            return errors;
        }
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "lua"))
                .collect(),
            Err(e) => {
                errors.push(format!("cannot read scripts dir '{}': {e}", dir.display()));
                return errors;
            }
        };
        entries.sort();
        for script_path in entries {
            if let Err(e) = self.lua_engine.load_script(&script_path) {
                errors.push(e);
            }
        }
        errors
    }
}
