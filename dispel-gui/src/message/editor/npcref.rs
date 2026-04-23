#[derive(Debug, Clone)]
pub enum NpcRefEditorMessage {
    LoadCatalog,
    NpcNamesLoaded(Result<Vec<(String, String)>, String>),
    Select(usize),
    AddEntry,
    RemoveEntry(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
}
