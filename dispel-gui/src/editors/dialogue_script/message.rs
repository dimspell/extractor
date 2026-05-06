use dispel_core::DialogueScript;

#[derive(Debug, Clone)]
pub enum DialogueScriptEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<DialogueScript>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
}
