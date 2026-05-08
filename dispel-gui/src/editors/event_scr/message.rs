use dispel_core::references::event_scr::{ActionFunction, EventScript, SpriteDefinition, Variable};

#[derive(Debug, Clone)]
pub enum EventScrEditorMessage {
    // Section switching
    SectionChanged(String),
    // Variable actions
    VariableAdded(usize, Variable),
    VariableEdited(usize, Variable),
    VariableDeleted(usize),
    // Sprite actions
    SpriteAdded(usize, SpriteDefinition),
    SpriteEdited(usize, SpriteDefinition),
    SpriteDeleted(usize),
    // Action function actions
    ActionAdded(usize, ActionFunction),
    ActionEdited(usize, ActionFunction),
    ActionDeleted(usize),
    // File operations
    Loaded(EventScript),
    LoadError(String),
    Saved,
    SaveError(String),
}
