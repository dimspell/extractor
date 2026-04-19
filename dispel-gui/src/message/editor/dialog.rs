use dispel_core::Dialog;

#[derive(Debug, Clone)]
pub enum DialogEditorMessage {
    SelectDialog(usize),
    FieldChanged(usize, String, String),
    Save,
    /// tab_id captured at task-spawn time so async result routes to the right editor.
    CatalogLoaded(usize, Result<Vec<Dialog>, String>),
}
