use dispel_core::ChData;

#[derive(Debug, Clone)]
pub enum ChDataEditorMessage {
    LoadCatalog,
    SelectData(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    CatalogLoaded(Result<Vec<ChData>, String>),
}
