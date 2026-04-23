use dispel_core::EventNpcRef;

#[derive(Debug, Clone)]
pub enum EventNpcRefEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<EventNpcRef>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
