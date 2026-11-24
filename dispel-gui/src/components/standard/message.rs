use iced::widget::pane_grid;

/// Generic message type shared by all standard single-file catalog editors.
///
/// Each concrete editor creates a type alias, e.g.:
/// `pub type WeaponEditorMessage = StandardEditorMessage<WeaponItem>;`
#[derive(Debug, Clone)]
pub enum StandardEditorMessage<T: Clone + std::fmt::Debug + 'static> {
    LoadCatalog,
    CatalogLoaded(Result<Vec<T>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(pane_grid::ResizeEvent),
    PaneClicked(pane_grid::Pane),
    Save,
    /// The first field is the id of the tab that issued the save. It is
    /// captured when the save starts so a completed save can clear the right
    /// tab's dirty flag even after the user switches tabs.
    Saved(usize, Result<(), String>),
}
