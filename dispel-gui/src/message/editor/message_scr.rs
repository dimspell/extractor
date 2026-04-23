#[derive(Debug, Clone)]
pub enum MessageScrEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<dispel_core::Message>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
