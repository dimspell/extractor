use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ModPackagerMessage {
    BrowseFiles,
    FilesChosen(Vec<PathBuf>),
    AddFile(PathBuf),
    RemoveFile(usize),
    NameChanged(String),
    VersionChanged(String),
    AuthorChanged(String),
    DescriptionChanged(String),
    Export,
    Exported(Result<PathBuf, String>),
}
