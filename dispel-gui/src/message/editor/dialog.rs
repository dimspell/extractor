use dispel_core::Dialog;

#[derive(Debug, Clone)]
pub enum DialogEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<Dialog>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
}
