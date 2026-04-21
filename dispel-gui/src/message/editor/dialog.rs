use dispel_core::Dialog;

#[derive(Debug, Clone)]
pub enum DialogEditorMessage {
    ScanDialogs,
    Scanned(Result<Vec<Dialog>, String>),
    SelectDialog(usize),
    FieldChanged(usize, String, String),
    Save,
    Saved(Result<(), String>),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
}
