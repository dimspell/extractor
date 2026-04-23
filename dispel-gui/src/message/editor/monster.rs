#[derive(Debug, Clone)]
pub enum MonsterEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<dispel_core::Monster>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
}
