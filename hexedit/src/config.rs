use std::sync::Arc;

use iced::Task;

use super::domain::save::{SaveOutcome, SaveRequest};
use super::domain::write_mode::{EncodingEntry, WriteMode};
use super::state::HexEditorState;
use super::ui::inspector::InspectorEntry;

/// Host-facing port for persisting the open hex document.
///
/// The host receives a domain-level [`SaveRequest`] (a path plus the current
/// bytes) and returns a task producing a [`SaveOutcome`]. No hexedit message
/// types cross this boundary — the editor maps `SaveOutcome` back onto its
/// own internal message after the task completes. `iced::Task` is used purely
/// as the async delivery mechanism, as it is throughout this Iced widget.
pub type SavePort = Arc<dyn Fn(SaveRequest) -> Task<SaveOutcome> + Send + Sync>;

/// Host-facing port for persisting the selected write mode.
///
/// Carries only the domain-level [`WriteMode`] value. In particular, it does
/// **not** leak `HexEditorMessage` (or any other internal message type) into
/// the host: changing the write mode is a synchronous host side-effect, not an
/// async editor operation.
pub type WriteModeCallback = Arc<dyn Fn(WriteMode) + Send + Sync>;

/// External configuration injected into the hex editor by the host application.
///
/// Assemble this with [`HexEditorConfig::new`] and its chainable setters. Both
/// embedders (the GUI and the standalone binary) use the same builder so no
/// field can be silently dropped by one of them.
pub struct HexEditorConfig {
    /// Halloy-style gap between panes in the pane grid (in logical pixels).
    /// Default: 4. Set to 0 to disable spacing.
    pub pane_gap: u16,
    /// Optional save-to-mod port. `None` hides the save button.
    pub on_save: Option<SavePort>,
    /// Label for the save button (e.g. "Save into `my-mod`").
    pub save_label: String,
    /// True when all prerequisites for saving are met (mod session active,
    /// file is inside game directory, etc.).
    pub can_save: bool,
    /// Contextual hint shown next to the save button explaining why it's disabled
    /// (e.g. "no recording active", "set a game directory").
    pub save_hint: String,
    /// Additional inspector entries from host-specific decoders.
    pub extra_entries: Vec<InspectorEntry>,
    /// User‑defined custom text encodings (persisted by the host).
    pub custom_encodings: Vec<EncodingEntry>,
    /// Optional callback fired when the write mode changes. The host can use
    /// this to persist the selection; it receives only the domain value.
    pub on_write_mode_changed: Option<WriteModeCallback>,
}

impl Default for HexEditorConfig {
    fn default() -> Self {
        Self {
            pane_gap: 4,
            on_save: None,
            save_label: String::new(),
            can_save: false,
            save_hint: String::new(),
            extra_entries: Vec::new(),
            custom_encodings: Vec::new(),
            on_write_mode_changed: None,
        }
    }
}

impl HexEditorConfig {
    /// Start from the canonical defaults. Use the chainable setters below to
    /// complete the configuration.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pane_gap(mut self, gap: u16) -> Self {
        self.pane_gap = gap;
        self
    }

    pub fn on_save(mut self, port: SavePort) -> Self {
        self.on_save = Some(port);
        self
    }

    /// Optional variant of [`HexEditorConfig::on_save`] for hosts whose save
    /// callback depends on prerequisites that may be missing.
    pub fn on_save_optional(mut self, port: Option<SavePort>) -> Self {
        self.on_save = port;
        self
    }

    pub fn with_save_label(mut self, label: impl Into<String>) -> Self {
        self.save_label = label.into();
        self
    }

    pub fn can_save(mut self, can_save: bool) -> Self {
        self.can_save = can_save;
        self
    }

    pub fn save_hint(mut self, hint: impl Into<String>) -> Self {
        self.save_hint = hint.into();
        self
    }

    pub fn extra_entries(mut self, entries: Vec<InspectorEntry>) -> Self {
        self.extra_entries = entries;
        self
    }

    pub fn custom_encodings(mut self, encodings: Vec<EncodingEntry>) -> Self {
        self.custom_encodings = encodings;
        self
    }

    pub fn on_write_mode_changed(mut self, callback: WriteModeCallback) -> Self {
        self.on_write_mode_changed = Some(callback);
        self
    }

    /// Optional variant of [`HexEditorConfig::on_write_mode_changed`], used by
    /// hosts that build the callback conditionally.
    pub fn on_write_mode_changed_callback(mut self, callback: Option<WriteModeCallback>) -> Self {
        self.on_write_mode_changed = callback;
        self
    }

    pub fn save_label(&self) -> &str {
        if self.save_label.is_empty() {
            "Save"
        } else {
            &self.save_label
        }
    }

    pub fn has_save(&self) -> bool {
        self.on_save.is_some()
    }

    pub fn can_save_now(&self, state: &HexEditorState) -> bool {
        self.can_save && self.on_save.is_some() && state.provider.dirty_count() > 0
    }
}

/// Shared assembly for the standalone `hexedit` binary.
///
/// The GUI has its own prerequisite rules (recording / game directory), but
/// both hosts go through [`HexEditorConfig::new`] so the common fields —
/// `pane_gap`, the default save label, the custom-encoding list, and the
/// write-mode port — cannot drift apart. Keeping this helper in the library
/// also makes the standalone assembly testable without launching the binary.
pub fn standalone_config(
    dirty: bool,
    custom_encodings: Vec<EncodingEntry>,
    extra_entries: Vec<InspectorEntry>,
    on_save: SavePort,
    on_write_mode_changed: Option<WriteModeCallback>,
) -> HexEditorConfig {
    HexEditorConfig::new()
        .pane_gap(4)
        .on_save(on_save)
        .with_save_label("Save")
        .can_save(true)
        .save_hint(if dirty {
            String::new()
        } else {
            "  ·  no edits".to_string()
        })
        .extra_entries(extra_entries)
        .custom_encodings(custom_encodings)
        .on_write_mode_changed_callback(on_write_mode_changed)
}
