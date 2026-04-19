use dispel_core::DialogueText;

#[derive(Debug, Clone)]
pub enum DialogueTextEditorMessage {
    SelectText(usize),
    FieldChanged(usize, String, String),
    TextAction(usize, iced::widget::text_editor::Action),
    CommentAction(usize, iced::widget::text_editor::Action),
    Save,
    /// tab_id captured at task-spawn time so async result routes to the right editor.
    CatalogLoaded(usize, Result<Vec<DialogueText>, String>),
}
