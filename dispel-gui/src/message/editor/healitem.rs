#[derive(Debug, Clone)]
pub enum HealItemEditorMessage {
    BrowseSpritePath,
    ScanItems,
    Scanned(Result<Vec<dispel_core::HealItem>, String>),
    SelectItem(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
}
