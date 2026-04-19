use dispel_core::DrawItem;

#[derive(Debug, Clone)]
pub enum DrawItemEditorMessage {
    LoadCatalog,
    SelectItem(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    CatalogLoaded(Result<Vec<DrawItem>, String>),
}
