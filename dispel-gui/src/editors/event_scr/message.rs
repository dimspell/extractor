use dispel_core::references::event_scr::EventScript;
use std::path::PathBuf;
use super::state::SectionTab;

#[derive(Debug, Clone)]
pub enum EventScrEditorMessage {
    // Section switching
    SectionChanged(SectionTab),
    // Variable actions
    VariableAdded,
    VariableNameChanged(usize, String),
    VariableValueChanged(usize, String),
    VariableDeleted(usize),
    // MAP/CHR/NPC/WAV line actions
    LineAdded(SectionTab),
    LineContentChanged(SectionTab, usize, String),
    LineDeleted(SectionTab, usize),
    // Sprite actions
    SpriteAdded,
    SpriteAliasChanged(usize, String),
    SpriteFileChanged(usize, String),
    SpriteDeleted(usize),
    // Action function actions
    ActionAdded,
    ActionRawAdded,
    ActionPrefixChanged(usize, String),
    ActionFunctionChanged(usize, String),
    ActionParamsChanged(usize, String),
    ActionRawContentChanged(usize, String),
    ActionDeleted(usize),
    // File operations
    LoadScript(PathBuf),
    ScriptLoaded(EventScript),
    LoadError(String),
    SaveScript,
    SaveSuccess,
    SaveError(String),
}
