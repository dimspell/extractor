use std::path::PathBuf;

/// Messages from the file tree.
#[derive(Debug, Clone)]
pub enum FileTreeMessage {
    ToggleDir(PathBuf),
    OpenFile(PathBuf),
    Search(String),
    /// Context menu actions
    ExtractToJson(PathBuf),
    ValidateFile(PathBuf),
    ShowInFileManager(PathBuf),
}
