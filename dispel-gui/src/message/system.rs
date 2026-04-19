use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum SystemMessage {
    CloseRequested,
    CloseApp,
    Undo,
    Redo,
    Save,
    IndexLoaded(Result<crate::search_index::SearchIndex, String>),
    CacheIndexationComplete(crate::file_index_cache::FileIndexCache),
    CacheIndexationFailed,
    IndexSaveRequested,
    IndexComplete,
    IndexSaveComplete,
    ClearLog,
    ToggleAutoSave,
    CheckDraftConflicts,
    ApplyDraft(String),
    DiscardDraft(String),
    RebuildIndex,
    ClearWorkspace,
    BrowseSharedGamePath,
    FileSelected {
        field: String,
        path: Option<PathBuf>,
    },
    ShowError(String),
    DismissError,
}
