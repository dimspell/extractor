#[derive(Debug, Clone)]
pub enum MonsterRefEditorMessage {
    SelectEntry(usize),
    AddEntry,
    RemoveEntry(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    LoadCatalog(std::path::PathBuf),
    LoadMonsterNames,
    MonsterNamesLoaded(Result<Vec<(String, String)>, String>),
}
