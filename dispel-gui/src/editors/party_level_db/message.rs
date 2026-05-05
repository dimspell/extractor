#[derive(Debug, Clone)]
pub enum PartyLevelDbEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<(Vec<dispel_core::PartyLevelNpc>, Vec<dispel_core::PartyRef>), String>),
    SelectNpc(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
