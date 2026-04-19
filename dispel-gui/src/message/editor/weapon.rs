#[derive(Debug, Clone)]
pub enum WeaponEditorMessage {
    ScanWeapons,
    Scanned(Result<Vec<dispel_core::WeaponItem>, String>),
    SelectWeapon(usize),
    FieldChanged(usize, String, String),
    Save,
    Saved(Result<(), String>),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
}
