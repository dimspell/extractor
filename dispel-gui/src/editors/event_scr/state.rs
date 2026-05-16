use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::loading_state::LoadingState;
use dispel_core::references::event_scr::EventScript;

use super::functions::{EventScriptFunctionIndex, IndexProgress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SectionTab {
    #[default]
    Header,
    Var,
    Map,
    Chr,
    Npc,
    Spr,
    Wav,
    Act,
}

impl SectionTab {
    pub fn label(&self) -> &'static str {
        match self {
            SectionTab::Header => "Header",
            SectionTab::Var => "Variables",
            SectionTab::Map => "Map",
            SectionTab::Chr => "Character",
            SectionTab::Npc => "NPCs",
            SectionTab::Spr => "Sprites",
            SectionTab::Wav => "Sounds",
            SectionTab::Act => "ACT",
        }
    }
}

/// Status of the function-index background scan.
#[derive(Debug, Clone, Default)]
pub enum FunctionIndexState {
    #[default]
    Idle,
    Indexing {
        progress: Arc<IndexProgress>,
    },
    Loaded(EventScriptFunctionIndex),
    Failed(String),
}

#[derive(Debug, Clone, Default)]
pub struct EventScriptEditorState {
    pub script_loading: LoadingState<EventScript>,
    pub file_path: Option<PathBuf>,
    pub panels_expanded: HashSet<SectionTab>,
    pub modified: bool,
    pub save_error: Option<String>,
    pub act_parse_errors: Vec<(usize, String)>,
    pub act_folded: HashSet<usize>,
    pub index_state: FunctionIndexState,
    pub picker_open: bool,
    pub picker_filter: String,
    pub pending_block_insert: Option<usize>,
}

impl EventScriptEditorState {
    pub fn is_loaded(&self) -> bool {
        matches!(self.script_loading, LoadingState::Loaded(_))
    }
}
