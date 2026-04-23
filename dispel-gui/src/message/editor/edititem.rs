use dispel_core::EditItem;

#[derive(Debug, Clone)]
pub enum EditItemEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<EditItem>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,    
}
