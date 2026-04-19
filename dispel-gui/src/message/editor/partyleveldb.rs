#[derive(Debug, Clone)]
pub enum PartyLevelDbEditorMessage {
    LoadCatalog,
    SelectRecord(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    CatalogLoaded(Result<Vec<dispel_core::PartyLevelNpc>, String>),
}
