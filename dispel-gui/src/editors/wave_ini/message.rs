use dispel_core::WaveIni;

#[derive(Debug, Clone)]
pub enum WaveIniEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<WaveIni>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
    ExportWav(usize),
    ExportedWav(Result<String, String>),
}
