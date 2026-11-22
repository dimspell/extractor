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

    /// The line vectors shown as editable body rows, if this section has any.
    pub fn body_content<'a>(&self, script: &'a EventScript) -> Option<&'a Vec<String>> {
        match self {
            SectionTab::Map => Some(&script.map_content),
            SectionTab::Chr => Some(&script.chr_content),
            SectionTab::Npc => Some(&script.npc_content),
            SectionTab::Wav => Some(&script.wav_content),
            SectionTab::Header | SectionTab::Var | SectionTab::Spr | SectionTab::Act => None,
        }
    }

    /// Mutable counterpart of [`SectionTab::body_content`].
    pub fn body_content_mut<'a>(&self, script: &'a mut EventScript) -> Option<&'a mut Vec<String>> {
        match self {
            SectionTab::Map => Some(&mut script.map_content),
            SectionTab::Chr => Some(&mut script.chr_content),
            SectionTab::Npc => Some(&mut script.npc_content),
            SectionTab::Wav => Some(&mut script.wav_content),
            SectionTab::Header | SectionTab::Var | SectionTab::Spr | SectionTab::Act => None,
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
    pub save_warning: Option<String>,
    pub act_parse_errors: Vec<(usize, String)>,
    pub act_folded: HashSet<usize>,
    pub index_state: FunctionIndexState,
    pub picker_open: bool,
    pub picker_filter: String,
    pub pending_block_insert: Option<usize>,
    pub suggestion_visible: bool,
    pub suggestion_active_index: Option<usize>,
    pub focused_action_index: Option<usize>,
}

impl EventScriptEditorState {
    pub fn is_loaded(&self) -> bool {
        matches!(self.script_loading, LoadingState::Loaded(_))
    }
}
