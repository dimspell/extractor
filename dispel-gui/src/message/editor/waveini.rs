use dispel_core::WaveIni;

#[derive(Debug, Clone)]
pub enum WaveIniEditorMessage {
    LoadCatalog,
    Scanned(Result<Vec<WaveIni>, String>),
    SelectWave(usize),
    FieldChanged(usize, String, String),
    Save,
    Saved(Result<(), String>),
    ExportWav(usize),
    ExportedWav(Result<String, String>),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
}
