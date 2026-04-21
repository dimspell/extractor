use crate::view::editor::SpreadsheetMessage;
use dispel_core::DialogueText;
use iced::widget::pane_grid::ResizeEvent;

#[derive(Debug, Clone)]
pub enum DialogueTextEditorMessage {
    ScanDialogueTexts,
    SelectText(usize),
    FieldChanged(usize, String, String),
    TextAction(usize, iced::widget::text_editor::Action),
    CommentAction(usize, iced::widget::text_editor::Action),
    Save,
    Spreadsheet(SpreadsheetMessage),
    PaneResized(ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    /// tab_id captured at task-spawn time so async result routes to the right editor.
    CatalogLoaded(usize, Result<Vec<DialogueText>, String>),
}
