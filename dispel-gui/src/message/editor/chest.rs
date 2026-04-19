#[derive(Debug, Clone)]
pub enum ChestEditorMessage {
    ScanMaps,
    MapsScanned(Result<Vec<String>, String>),
    LoadCatalog,
    CatalogLoaded(Result<crate::state::chest_editor::ItemCatalog, String>),
    SelectMap,
    SelectMapFromFile(String),
    MapLoaded(Result<Vec<(usize, dispel_core::ExtraRef)>, String>),
    SelectChest(usize),
    FieldChanged(usize, String, String),
    Save,
    Saved(Result<(), String>),
    Add,
    Delete(usize),
}
