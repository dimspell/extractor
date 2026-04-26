use crate::loading_state::LoadingState;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct ModMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub struct ModPackagerState {
    pub selected_files: Vec<PathBuf>,
    pub metadata: ModMetadata,
    pub status_msg: String,
    pub loading_state: LoadingState<()>,
}
