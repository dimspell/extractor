use std::path::PathBuf;

use dispel_core::modding::{ChangeAction, Conflict, InstalledMod, ModManifest};

use crate::components::loading_state::LoadingState;

/// Top-level tab inside the Mod Manager editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModManagerTab {
    #[default]
    Library,
    Detail,
    Conflicts,
}

/// Legacy struct kept for `localization_manager`. New code should use
/// [`dispel_core::modding::ModManifest`] directly.
#[derive(Debug, Default, Clone)]
pub struct ModMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

/// Root state for the Mod Manager editor.
#[derive(Debug, Default)]
pub struct ModPackagerState {
    pub tab: ModManagerTab,

    /// Workspace root (`<root>/mods/`, `<root>/vanilla/`, `<root>/enabled.json`).
    /// `None` until the user picks one (or one is auto-derived from the
    /// configured game path on first open).
    pub workspace_root: Option<PathBuf>,

    /// Cached library list, in display order (enabled first).
    pub mods: Vec<InstalledMod>,

    /// Currently selected mod (Detail tab).
    pub selected_slug: Option<String>,
    /// Loaded manifest for the selected mod.
    pub selected_manifest: Option<ModManifest>,
    /// Loaded change-log timeline for the selected mod (read-only in v1).
    pub selected_changes: Vec<ChangeAction>,

    /// Manifest editor buffers.
    pub edit_name: String,
    pub edit_version: String,
    pub edit_author: String,
    pub edit_description: String,
    pub edit_dirty: bool,

    /// Cached conflict list across enabled mods. Recomputed on Refresh and
    /// after any load-order change.
    pub conflicts: Vec<Conflict>,

    pub status_msg: String,
    pub loading_state: LoadingState<()>,
}
