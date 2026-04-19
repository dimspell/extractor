#[derive(Debug, Clone)]
pub enum NpcRefEditorMessage {
    SelectNpc(usize),
    AddEntry,
    FieldChanged(usize, String, String),
    RemoveEntry(usize),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    LoadNpcNames,
    NpcNamesLoaded(Result<Vec<(String, String)>, String>),
}
