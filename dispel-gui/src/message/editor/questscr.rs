use dispel_core::Quest;

#[derive(Debug, Clone)]
pub enum QuestScrEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<Quest>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    DescriptionAction(usize, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
