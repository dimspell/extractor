#[derive(Debug, Clone)]
pub enum MonsterEditorMessage {
    LoadCatalog,
    ScanMonsters,
    Scanned(Result<Vec<dispel_core::Monster>, String>),
    SelectMonster(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
}
