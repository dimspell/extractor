use crate::components::loading_state::LoadingState;
use dispel_core::references::event_scr::EventScript;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
            SectionTab::Var => "VAR",
            SectionTab::Map => "MAP",
            SectionTab::Chr => "CHR",
            SectionTab::Npc => "NPC",
            SectionTab::Spr => "SPR",
            SectionTab::Wav => "WAV",
            SectionTab::Act => "ACT",
        }
    }

    pub fn all() -> Vec<SectionTab> {
        vec![
            SectionTab::Header,
            SectionTab::Var,
            SectionTab::Map,
            SectionTab::Chr,
            SectionTab::Npc,
            SectionTab::Spr,
            SectionTab::Wav,
            SectionTab::Act,
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventScriptEditorState {
    pub script_loading: LoadingState<EventScript>,
    pub file_path: Option<PathBuf>,
    pub active_section: SectionTab,
    pub modified: bool,
    pub save_error: Option<String>,
    pub act_parse_errors: Vec<(usize, String)>,
}

impl EventScriptEditorState {
    pub fn is_loaded(&self) -> bool {
        matches!(self.script_loading, LoadingState::Loaded(_))
    }
}
