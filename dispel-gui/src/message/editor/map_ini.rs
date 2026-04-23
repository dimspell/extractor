use dispel_core::MapIni;

#[derive(Debug, Clone)]
pub enum MapIniEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<MapIni>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
