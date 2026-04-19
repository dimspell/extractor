#[derive(Debug, Clone)]
pub enum ExtraRefEditorMessage {
    SelectItem(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    /// tab_id is captured at task-spawn time so the right editor is updated on async completion.
    CatalogLoaded(usize, Result<Vec<dispel_core::ExtraRef>, String>),
}
