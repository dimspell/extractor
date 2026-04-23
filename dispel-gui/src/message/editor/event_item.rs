use dispel_core::EventItem;

#[derive(Debug, Clone)]
pub enum EventItemEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<EventItem>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
