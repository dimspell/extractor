#[derive(Debug, Clone)]
pub enum PartyIniEditorMessage {
    LoadCatalog,
    ScanNpcs,
    SelectNpc(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    CatalogLoaded(Result<Vec<dispel_core::PartyIniNpc>, String>),
}
