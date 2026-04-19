use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum StartPageMessage {
    PathInputChanged(String),
    Browse,
    Continue,
    SelectRecentPath(PathBuf),
    BackToStart,
}
