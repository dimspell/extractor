use crate::view::editor::SpreadsheetMessage;
use dispel_core::DialogueParagraph;
use iced::widget::pane_grid::ResizeEvent;

#[derive(Debug, Clone)]
pub enum DialogueParagraphEditorMessage {
    ScanCatalog,
    /// tab_id captured at task-spawn time so async result routes to the right editor.
    CatalogLoaded(usize, Result<Vec<DialogueParagraph>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(SpreadsheetMessage),
    PaneResized(ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
