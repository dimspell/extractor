use dispel_core::PartyRef;

#[derive(Debug, Clone)]
pub enum PartyRefEditorMessage {
    LoadCatalog,
    ScanParty,
    SelectMember(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    CatalogLoaded(Result<Vec<PartyRef>, String>),
}
