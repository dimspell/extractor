use std::path::PathBuf;

/// Messages from the file tree.
#[derive(Debug, Clone)]
pub enum FileTreeMessage {
    ToggleDir(PathBuf),
    OpenFile(PathBuf),
    Search(String),
    /// Debounced search: the UI input has been idle long enough to apply the filter.
    ApplyDebouncedSearch(String),
    /// Context menu actions
    OpenAsHex(PathBuf),
    ExtractToJson(PathBuf),
    ValidateFile(PathBuf),
    ShowInFileManager(PathBuf),
}
