use crate::components::FileTree;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum SystemMessage {
    CloseRequested,
    CloseApp,
    Undo,
    Redo,
    Save,
    IndexLoaded(Result<crate::indexation::search_index::SearchIndex, String>),
    CacheIndexationComplete(crate::indexation::file_index_cache::FileIndexCache),
    CacheIndexationFailed,
    IndexSaveRequested,
    IndexComplete,
    IndexSaveComplete,
    ToggleAutoSave,
    CheckDraftConflicts,
    ApplyDraft(String),
    DiscardDraft(String),
    RebuildIndex,
    ClearWorkspace,
    BrowseSharedGamePath,
    FileTreeScanned(FileTree),
    FileSelected {
        field: String,
        path: Option<PathBuf>,
    },
    ShowError(String),
    DismissError,
}
