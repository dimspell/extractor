use super::state::SectionTab;
use crate::editors::event_scr::functions::EventScriptFunctionIndex;
use dispel_core::references::event_scr::EventScript;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum EventScrEditorMessage {
    // Panel toggling
    TogglePanel(SectionTab),
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
    ActionPrefixPicked(usize, Option<String>),
    ActionFunctionChanged(usize, String),
    ActionParamsChanged(usize, String),
    ActionRawContentChanged(usize, String),
    ActionDeleted(usize),
    // Tree view
    ToggleFold(usize),
    IfConditionChanged(usize, String),
    ReturnValueChanged(usize, String),
    // Reorder
    MoveActionUp(usize),
    MoveActionDown(usize),
    // Block insertion with function picker
    InsertWithPickerAt(usize),
    // File operations
    LoadScript(PathBuf),
    ScriptLoaded(EventScript),
    LoadError(String),
    SaveScript,
    SaveSuccess,
    SaveError(String),
    // Function index
    BuildFunctionIndex,
    FunctionIndexBuilt(Result<EventScriptFunctionIndex, String>),
    CancelIndexing,
    IndexTick,
    // Function picker
    ToggleFunctionPicker,
    PickerFilterChanged(String),
    InsertPickedFunction(String, usize),
    // Inline function suggestions
    SuggestionSelect(usize, String),
    SuggestionDismiss,
    // Insert helpers
    InsertIfBlock,
    InsertElseBlock,
    InsertReturnBlock,
    // Keyboard shortcuts
    KeyboardShortcut(KeyboardShortcut),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardShortcut {
    InsertActionBelow,
    TogglePicker,
    MoveActionUp,
    MoveActionDown,
}
