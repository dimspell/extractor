use std::path::PathBuf;
use std::sync::Arc;

use dispel_core::modding::{ChangeAction, Conflict, FieldKey, InstalledMod, ModManifest};

use super::recording::ObservedAction;
use super::state::ModManagerTab;

/// All Mod Manager messages. The variant set covers tab navigation,
/// workspace lifecycle, mod CRUD, manifest editing, load-order management,
/// apply/revert, and zip import/export.
#[derive(Debug, Clone)]
pub enum ModPackagerMessage {
    // Tabs
    TabSelected(ModManagerTab),

    // Workspace lifecycle
    OpenWorkspace,
    WorkspacePicked(Option<PathBuf>),
    Refresh,
    Refreshed(Result<LibrarySnapshot, String>),

    // Mod CRUD
    CreateMod,
    Created(Result<String, String>),
    SelectMod(String),
    Selected(Result<SelectedMod, String>),
    DeleteMod(String),
    Deleted(Result<(), String>),

    // Manifest editor
    NameChanged(String),
    VersionChanged(String),
    AuthorChanged(String),
    DescriptionChanged(String),
    SaveManifest,
    Saved(Result<(), String>),

    // Library actions
    ToggleEnabled(String),
    MoveUp(String),
    MoveDown(String),

    // Apply / revert
    Apply,
    Applied(Result<ApplyOutcome, String>),
    Revert,
    Reverted(Result<RevertOutcome, String>),

    // Recording
    StartRecording(String),
    StopRecording,
    RecordingObserved(ObservedAction),
    /// Fires after the debounce interval elapses for one pending edit.
    /// Only the timer matching the latest `generation` is allowed to flush;
    /// stale timers are dropped silently.
    RecordingDebounceFired {
        key: crate::state::RecordingKey,
        generation: u64,
    },
    RecordingPersisted(Result<(), String>),

    // Conflict resolution (per-field pin)
    PinConflict { key: FieldKey, mod_slug: String },
    UnpinConflict { key: FieldKey },

    // Import / export
    ImportZip,
    ImportPicked(Option<PathBuf>),
    Imported(Result<String, String>),
    ExportZip(String),
    ExportPicked(String, Option<PathBuf>),
    Exported(Result<PathBuf, String>),
}

/// Payload for [`ModPackagerMessage::Refreshed`] — installed mods and the
/// derived conflicts list, fetched in one workspace round-trip.
#[derive(Debug, Clone)]
pub struct LibrarySnapshot {
    pub mods: Vec<InstalledMod>,
    pub conflicts: Vec<Conflict>,
}

/// Payload for [`ModPackagerMessage::Selected`] — keeps the message clonable
/// despite carrying a fully-loaded change log.
#[derive(Debug, Clone)]
pub struct SelectedMod {
    pub slug: String,
    pub manifest: ModManifest,
    pub changes: Arc<Vec<ChangeAction>>,
}

/// Summary of a successful apply, surfaced into the status bar.
#[derive(Debug, Clone)]
pub struct ApplyOutcome {
    pub actions_applied: usize,
    pub written: usize,
    pub deleted: usize,
}

#[derive(Debug, Clone)]
pub struct RevertOutcome {
    pub restored: usize,
}
