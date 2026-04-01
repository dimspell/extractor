use crate::chest_editor;
use crate::db;
use crate::types::{DbOp, MapOp, RefOp, SpriteMode, Tab};
use std::path::PathBuf;

use dispel_core::WeaponItem;

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    // Map
    MapOpSelected(MapOp),
    MapInputChanged(String),
    MapOutputChanged(String),
    MapMapPathChanged(String),
    MapBtlPathChanged(String),
    MapGtlPathChanged(String),
    MapSaveSpritesToggled(bool),
    MapDatabaseChanged(String),
    MapMapIdChanged(String),
    MapGtlAtlasChanged(String),
    MapBtlAtlasChanged(String),
    MapAtlasColumnsChanged(String),
    MapGamePathChanged(String),
    // Browse buttons
    BrowseMapInput,
    BrowseMapMapPath,
    BrowseMapBtlPath,
    BrowseMapGtlPath,
    BrowseMapGtlAtlas,
    BrowseMapBtlAtlas,
    BrowseMapGamePath,
    BrowseRefInput,
    BrowseSpriteInput,
    BrowseSoundInput,
    BrowseSoundOutput,
    BrowseExtractorPath,
    FileSelected {
        field: String,
        path: Option<PathBuf>,
    },
    // Ref
    RefOpSelected(RefOp),
    RefInputChanged(String),
    // Database
    DbOpSelected(DbOp),
    // Sprite
    SpriteInputChanged(String),
    SpriteModeSelected(SpriteMode),
    // Sound
    SoundInputChanged(String),
    SoundOutputChanged(String),
    // Global
    ExtractorPathChanged(String),
    Run,
    CommandFinished(Result<String, String>),
    ClearLog,
    // DB Viewer messages
    ViewerDbPathChanged(String),
    ViewerBrowseDb,
    ViewerConnect,
    ViewerTablesLoaded(Result<Vec<String>, String>),
    ViewerSelectTable(String),
    ViewerDataLoaded(Result<db::QueryResult, String>),
    ViewerSearch(String),
    ViewerSortColumn(usize),
    ViewerNextPage,
    ViewerPrevPage,
    ViewerCellClick(usize, usize),
    ViewerCellEdit(String),
    ViewerCellConfirm,
    ViewerCellCancel,
    ViewerCommit,
    ViewerCommitDone(Result<usize, String>),
    ViewerToggleSql,
    ViewerSqlChanged(String),
    ViewerRunSql,
    ViewerExportCsv,
    ViewerCsvSaved(Result<String, String>),
    ViewerRevertEdits,
    // Chest Editor internal
    ChestCatalogLoaded(Result<chest_editor::ItemCatalog, String>),
    // Weapon Editor internal
    WeaponCatalogLoaded(Result<Vec<WeaponItem>, String>),
    ChestMapLoaded(Result<Vec<dispel_core::ExtraRef>, String>),
    ChestMapsScanned(Result<Vec<PathBuf>, String>),
    ChestSaved(Result<(), String>),
    // Chest Editor
    ChestOpBrowseGamePath,
    ChestOpBrowseMapFile,
    ChestOpScanMaps,
    ChestOpLoadCatalog,
    ChestOpSelectMap,
    ChestOpSelectMapFromFile(PathBuf),
    ChestOpSelectChest(usize),
    ChestOpFieldChanged(usize, String, String), // (original_index, field_name, new_value)
    ChestOpSave,
    ChestOpAdd,
    ChestOpDelete(usize),
    // Weapon Editor
    WeaponOpBrowseGamePath,
    WeaponOpLoadCatalog,
    WeaponOpScanWeapons,
    WeaponOpSelectWeapon(usize),
    WeaponOpFieldChanged(usize, String, String),
    WeaponOpSave,
}
