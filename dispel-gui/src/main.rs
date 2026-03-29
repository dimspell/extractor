use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable, text,
    text_input, toggler, vertical_space,
};
use iced::{color, Element, Fill, Font, Length, Task, Theme};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod chest_editor;
mod db;
mod style;
use dispel_core::commands::{self, Command, CommandFactory};
use dispel_core::Extractor;

pub fn main() -> iced::Result {
    iced::application("Dispel Extractor", App::update, App::view)
        .theme(|_| {
            Theme::custom(
                "Dispel Dark".into(),
                iced::theme::Palette {
                    background: color!(0x1a1a2e),
                    text: color!(0xe0e0e0),
                    primary: color!(0x6c63ff),
                    success: color!(0x2ecc71),
                    danger: color!(0xe74c3c),
                },
            )
        })
        .window_size((1100.0, 800.0))
        .run_with(App::new)
}

// ─── Tab enum ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Map,
    Ref,
    Database,
    Sprite,
    Sound,
    DbViewer,
    ChestEditor,
}

impl Tab {
    const ALL: &'static [Tab] = &[
        Tab::Map,
        Tab::Ref,
        Tab::Database,
        Tab::Sprite,
        Tab::Sound,
        Tab::DbViewer,
        Tab::ChestEditor,
    ];
    fn label(&self) -> &str {
        match self {
            Tab::Map => "Map",
            Tab::Ref => "Ref",
            Tab::Database => "Database",
            Tab::Sprite => "Sprite",
            Tab::Sound => "Sound",
            Tab::DbViewer => "DbViewer",
            Tab::ChestEditor => "ChestEditor",
        }
    }
}

// ─── Map sub-operations ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapOp {
    Tiles,
    Atlas,
    Render,
    FromDb,
    ToDb,
    Sprites,
}

impl MapOp {
    const ALL: &'static [MapOp] = &[
        MapOp::Tiles,
        MapOp::Atlas,
        MapOp::Render,
        MapOp::FromDb,
        MapOp::ToDb,
        MapOp::Sprites,
    ];
    fn label(&self) -> &str {
        match self {
            MapOp::Tiles => "Extract Tiles",
            MapOp::Atlas => "Generate Atlas",
            MapOp::Render => "Render Map",
            MapOp::FromDb => "Render from DB",
            MapOp::ToDb => "Import to DB",
            MapOp::Sprites => "Extract Sprites",
        }
    }
}
impl std::fmt::Display for MapOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ─── Ref sub-operations ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefOp {
    AllMaps,
    Map,
    Extra,
    Event,
    Monster,
    Npc,
    Wave,
    PartyRef,
    DrawItem,
    PartyPgp,
    PartyDialog,
    Dialog,
    Weapons,
    Monsters,
    MultiMagic,
    Store,
    NpcRef,
    MonsterRef,
    MiscItem,
    HealItems,
    ExtraRef,
    EventItems,
    EditItems,
    PartyLevel,
    PartyIni,
    EventNpcRef,
    Magic,
    Quest,
    Message,
    ChData,
}

impl RefOp {
    const ALL: &'static [RefOp] = &[
        RefOp::AllMaps,
        RefOp::Map,
        RefOp::Extra,
        RefOp::Event,
        RefOp::Monster,
        RefOp::Npc,
        RefOp::Wave,
        RefOp::PartyRef,
        RefOp::DrawItem,
        RefOp::PartyPgp,
        RefOp::PartyDialog,
        RefOp::Dialog,
        RefOp::Weapons,
        RefOp::Monsters,
        RefOp::MultiMagic,
        RefOp::Store,
        RefOp::NpcRef,
        RefOp::MonsterRef,
        RefOp::MiscItem,
        RefOp::HealItems,
        RefOp::ExtraRef,
        RefOp::EventItems,
        RefOp::EditItems,
        RefOp::PartyLevel,
        RefOp::PartyIni,
        RefOp::EventNpcRef,
        RefOp::Magic,
        RefOp::Quest,
        RefOp::Message,
        RefOp::ChData,
    ];
    fn cli_name(&self) -> &str {
        match self {
            RefOp::AllMaps => "all-maps",
            RefOp::Map => "map",
            RefOp::Extra => "extra",
            RefOp::Event => "event",
            RefOp::Monster => "monster",
            RefOp::Npc => "npc",
            RefOp::Wave => "wave",
            RefOp::PartyRef => "party-ref",
            RefOp::DrawItem => "draw-item",
            RefOp::PartyPgp => "party-pgp",
            RefOp::PartyDialog => "party-dialog",
            RefOp::Dialog => "dialog",
            RefOp::Weapons => "weapons",
            RefOp::Monsters => "monsters",
            RefOp::MultiMagic => "multi-magic",
            RefOp::Store => "store",
            RefOp::NpcRef => "npc-ref",
            RefOp::MonsterRef => "monster-ref",
            RefOp::MiscItem => "misc-item",
            RefOp::HealItems => "heal-items",
            RefOp::ExtraRef => "extra-ref",
            RefOp::EventItems => "event-items",
            RefOp::EditItems => "edit-items",
            RefOp::PartyLevel => "party-level",
            RefOp::PartyIni => "party-ini",
            RefOp::EventNpcRef => "event-npc-ref",
            RefOp::Magic => "magic",
            RefOp::Quest => "quest",
            RefOp::Message => "message",
            RefOp::ChData => "ch-data",
        }
    }
}
impl std::fmt::Display for RefOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cli_name())
    }
}

// ─── Database sub-operations ────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbOp {
    Import,
    DialogTexts,
    Maps,
    Databases,
    Refs,
    Rest,
}

impl DbOp {
    const ALL: &'static [DbOp] = &[
        DbOp::Import,
        DbOp::DialogTexts,
        DbOp::Maps,
        DbOp::Databases,
        DbOp::Refs,
        DbOp::Rest,
    ];
    fn label(&self) -> &str {
        match self {
            DbOp::Import => "Import All",
            DbOp::DialogTexts => "Dialog Texts",
            DbOp::Maps => "Maps",
            DbOp::Databases => "Databases",
            DbOp::Refs => "Refs",
            DbOp::Rest => "Rest",
        }
    }
    fn cli_name(&self) -> &str {
        match self {
            DbOp::Import => "import",
            DbOp::DialogTexts => "dialog-texts",
            DbOp::Maps => "maps",
            DbOp::Databases => "databases",
            DbOp::Refs => "refs",
            DbOp::Rest => "rest",
        }
    }
}
impl std::fmt::Display for DbOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ─── Sprite mode ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpriteMode {
    Sprite,
    Animation,
}
impl SpriteMode {
    const ALL: &'static [SpriteMode] = &[SpriteMode::Sprite, SpriteMode::Animation];
}
impl std::fmt::Display for SpriteMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpriteMode::Sprite => write!(f, "Sprite"),
            SpriteMode::Animation => write!(f, "Animation"),
        }
    }
}

// ─── Database Viewer state ──────────────────────────────────────────────────
const PAGE_SIZE: usize = 200;

struct DbViewerState {
    db_path: String,
    tables: Vec<String>,
    active_table: Option<String>,
    columns: Vec<db::ColumnInfo>,
    rows: Vec<Vec<String>>,
    total_rows: usize,
    page: usize,
    search: String,
    sort_col: Option<usize>,
    sort_dir: db::SortDir,
    editing_cell: Option<(usize, usize)>,
    edit_buffer: String,
    pending_edits: db::PendingEdits,
    sql_mode: bool,
    sql_query: String,
    status_msg: String,
    is_loading: bool,
}

impl Default for DbViewerState {
    fn default() -> Self {
        Self {
            db_path: String::from("database.sqlite"),
            tables: vec![],
            active_table: None,
            columns: vec![],
            rows: vec![],
            total_rows: 0,
            page: 0,
            search: String::new(),
            sort_col: None,
            sort_dir: db::SortDir::Asc,
            editing_cell: None,
            edit_buffer: String::new(),
            pending_edits: HashMap::new(),
            sql_mode: false,
            sql_query: String::new(),
            status_msg: String::new(),
            is_loading: false,
        }
    }
}

// ─── Application state ─────────────────────────────────────────────────────
struct App {
    active_tab: Tab,
    // Map fields
    map_op: Option<MapOp>,
    map_input: String,
    map_output: String,
    map_map_path: String,
    map_btl_path: String,
    map_gtl_path: String,
    map_save_sprites: bool,
    map_database: String,
    map_map_id: String,
    map_gtl_atlas: String,
    map_btl_atlas: String,
    map_atlas_columns: String,
    map_game_path: String,
    // Ref fields
    ref_op: Option<RefOp>,
    ref_input: String,
    // Database fields
    db_op: Option<DbOp>,
    // Sprite fields
    sprite_input: String,
    sprite_mode: Option<SpriteMode>,
    // Sound fields
    sound_input: String,
    sound_output: String,
    // Global
    extractor_path: String,
    log: String,
    is_running: bool,
    // DB Viewer
    viewer: Box<DbViewerState>,
    // Chest Editor
    chest_editor: Box<chest_editor::ChestEditorState>,
}

// ─── Messages ───────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
enum Message {
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
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                active_tab: Tab::Map,
                map_op: Some(MapOp::Render),
                map_input: String::new(),
                map_output: String::from("map.png"),
                map_map_path: String::new(),
                map_btl_path: String::new(),
                map_gtl_path: String::new(),
                map_save_sprites: false,
                map_database: String::from("database.sqlite"),
                map_map_id: String::new(),
                map_gtl_atlas: String::new(),
                map_btl_atlas: String::new(),
                map_atlas_columns: String::from("48"),
                map_game_path: String::new(),
                ref_op: Some(RefOp::AllMaps),
                ref_input: String::new(),
                db_op: Some(DbOp::Import),
                sprite_input: String::new(),
                sprite_mode: Some(SpriteMode::Sprite),
                sound_input: String::new(),
                sound_output: String::new(),
                extractor_path: String::from("dispel-extractor"),
                log: String::new(),
                is_running: false,
                viewer: Box::default(),
                chest_editor: Box::default(),
            },
            Task::none(),
        )
    }

    fn refresh_chests(&mut self) {
        let editor = &mut self.chest_editor;
        editor.filtered_chests = editor
            .all_records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.object_type == dispel_core::ExtraObjectType::Chest)
            .map(|(i, r)| (i, r.clone()))
            .collect();
    }

    fn load_map_file(&mut self, path: PathBuf) -> Task<Message> {
        self.chest_editor.is_loading = true;
        Task::perform(
            async move { dispel_core::ExtraRef::read_file(&path) },
            |res| Message::ChestMapLoaded(res.map_err(|e| e.to_string())),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            // Map
            Message::MapOpSelected(op) => {
                self.map_op = Some(op);
                Task::none()
            }
            Message::MapInputChanged(v) => {
                self.map_input = v;
                Task::none()
            }
            Message::MapOutputChanged(v) => {
                self.map_output = v;
                Task::none()
            }
            Message::MapMapPathChanged(v) => {
                self.map_map_path = v;
                Task::none()
            }
            Message::MapBtlPathChanged(v) => {
                self.map_btl_path = v;
                Task::none()
            }
            Message::MapGtlPathChanged(v) => {
                self.map_gtl_path = v;
                Task::none()
            }
            Message::MapSaveSpritesToggled(v) => {
                self.map_save_sprites = v;
                Task::none()
            }
            Message::MapDatabaseChanged(v) => {
                self.map_database = v;
                Task::none()
            }
            Message::MapMapIdChanged(v) => {
                self.map_map_id = v;
                Task::none()
            }
            Message::MapGtlAtlasChanged(v) => {
                self.map_gtl_atlas = v;
                Task::none()
            }
            Message::MapBtlAtlasChanged(v) => {
                self.map_btl_atlas = v;
                Task::none()
            }
            Message::MapAtlasColumnsChanged(v) => {
                self.map_atlas_columns = v;
                Task::none()
            }
            Message::MapGamePathChanged(v) => {
                self.map_game_path = v;
                Task::none()
            }
            // Browse buttons
            Message::BrowseMapInput => browse_file("map_input"),
            Message::BrowseMapMapPath => browse_file("map_map_path"),
            Message::BrowseMapBtlPath => browse_file("map_btl_path"),
            Message::BrowseMapGtlPath => browse_file("map_gtl_path"),
            Message::BrowseMapGtlAtlas => browse_file("map_gtl_atlas"),
            Message::BrowseMapBtlAtlas => browse_file("map_btl_atlas"),
            Message::BrowseMapGamePath => browse_folder("map_game_path"),
            Message::BrowseRefInput => browse_file("ref_input"),
            Message::BrowseSpriteInput => browse_file("sprite_input"),
            Message::BrowseSoundInput => browse_file("sound_input"),
            Message::BrowseSoundOutput => browse_file("sound_output"),
            Message::BrowseExtractorPath => browse_file("extractor_path"),
            Message::FileSelected { field, path } => {
                if let Some(p) = path {
                    let s = p.to_string_lossy().to_string();
                    match field.as_str() {
                        "map_input" => self.map_input = s,
                        "map_map_path" => self.map_map_path = s,
                        "map_btl_path" => self.map_btl_path = s,
                        "map_gtl_path" => self.map_gtl_path = s,
                        "map_gtl_atlas" => self.map_gtl_atlas = s,
                        "map_btl_atlas" => self.map_btl_atlas = s,
                        "map_game_path" => self.map_game_path = s,
                        "ref_input" => self.ref_input = s,
                        "sprite_input" => self.sprite_input = s,
                        "sound_input" => self.sound_input = s,
                        "sound_output" => self.sound_output = s,
                        "extractor_path" => self.extractor_path = s,
                        "viewer_db" => self.viewer.db_path = s,
                        "chest_game_path" => self.chest_editor.game_path = s,
                        "chest_map_file" => self.chest_editor.current_map_file = s,
                        _ => {}
                    }
                }
                Task::none()
            }
            // Ref
            Message::RefOpSelected(op) => {
                self.ref_op = Some(op);
                Task::none()
            }
            Message::RefInputChanged(v) => {
                self.ref_input = v;
                Task::none()
            }
            // Database
            Message::DbOpSelected(op) => {
                self.db_op = Some(op);
                Task::none()
            }
            // Sprite
            Message::SpriteInputChanged(v) => {
                self.sprite_input = v;
                Task::none()
            }
            Message::SpriteModeSelected(m) => {
                self.sprite_mode = Some(m);
                Task::none()
            }
            // Sound
            Message::SoundInputChanged(v) => {
                self.sound_input = v;
                Task::none()
            }
            Message::SoundOutputChanged(v) => {
                self.sound_output = v;
                Task::none()
            }
            // Global
            Message::ExtractorPathChanged(v) => {
                self.extractor_path = v;
                Task::none()
            }
            Message::Run => {
                let Some(cmd) = self.build_internal_command() else {
                    self.log
                        .push_str("⚠ No command configured or supported in GUI yet.\n");
                    return Task::none();
                };
                self.log.push_str(&format!(
                    "▸ Running internal command: {} [{}]\n",
                    cmd.name(),
                    cmd.description()
                ));
                self.is_running = true;

                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let result =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    cmd.execute()
                                }));
                            match result {
                                Ok(Ok(())) => Ok("Command finished successfully.\n".to_string()),
                                Ok(Err(e)) => Err(format!("Error: {}", e)),
                                Err(panic_err) => {
                                    if let Some(s) = panic_err.downcast_ref::<&str>() {
                                        Err(s.to_string())
                                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                                        Err(s.clone())
                                    } else {
                                        Err("Unknown panic occurred during execution".to_string())
                                    }
                                }
                            }
                        })
                        .await
                        .unwrap()
                    },
                    Message::CommandFinished,
                )
            }
            Message::CommandFinished(result) => {
                self.is_running = false;
                match result {
                    Ok(output) => {
                        self.log.push_str(&output);
                        self.log.push_str("✔ Done.\n\n");
                    }
                    Err(err) => {
                        self.log.push_str(&format!("✖ Error: {}\n\n", err));
                    }
                }
                Task::none()
            }
            Message::ClearLog => {
                self.log.clear();
                Task::none()
            }

            // ─── Chest Editor messages ──────────────────────────────
            Message::ChestOpBrowseGamePath => browse_folder("chest_game_path"),
            Message::ChestOpBrowseMapFile => browse_file("chest_map_file"),
            Message::ChestOpScanMaps => {
                if self.chest_editor.game_path.is_empty() {
                    self.chest_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.chest_editor.is_loading = true;
                let path = PathBuf::from(&self.chest_editor.game_path).join("ExtraInGame");
                Task::perform(
                    async move {
                        let mut files = vec![];
                        if let Ok(entries) = std::fs::read_dir(path) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() && p.extension().map(|e| e == "ref").unwrap_or(false)
                                {
                                    if p.file_name()
                                        .map(|n| n.to_string_lossy().starts_with("Ext"))
                                        .unwrap_or(false)
                                    {
                                        files.push(p);
                                    }
                                }
                            }
                        }
                        files.sort();
                        Ok(files)
                    },
                    |res| Message::ChestMapsScanned(res),
                )
            }
            Message::ChestMapsScanned(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(files) => {
                        self.chest_editor.map_files = files;
                        self.chest_editor.status_msg =
                            format!("Found {} map files.", self.chest_editor.map_files.len());
                    }
                    Err(e) => self.chest_editor.status_msg = format!("Error scanning maps: {}", e),
                }
                Task::none()
            }

            Message::ChestOpLoadCatalog => {
                if self.chest_editor.game_path.is_empty() {
                    self.chest_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.chest_editor.is_loading = true;
                let path = PathBuf::from(&self.chest_editor.game_path);
                Task::perform(
                    async move { chest_editor::ItemCatalog::load_from_folder(&path) },
                    |res| Message::ChestCatalogLoaded(res.map_err(|e| e.to_string())),
                )
            }
            Message::ChestCatalogLoaded(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.chest_editor.catalog = Some(catalog);
                        self.chest_editor.status_msg = "Catalog loaded successfully.".into();
                    }
                    Err(e) => {
                        self.chest_editor.status_msg = format!("Error loading catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::ChestOpSelectMap => {
                if self.chest_editor.current_map_file.is_empty() {
                    self.chest_editor.status_msg = "No map file selected.".into();
                    return Task::none();
                }
                self.load_map_file(PathBuf::from(&self.chest_editor.current_map_file))
            }
            Message::ChestOpSelectMapFromFile(path) => {
                self.chest_editor.current_map_file = path.to_string_lossy().to_string();
                self.load_map_file(path)
            }
            Message::ChestMapLoaded(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(records) => {
                        self.chest_editor.all_records = records;
                        self.chest_editor.status_msg = "Map loaded successfully.".into();
                        self.refresh_chests();
                    }
                    Err(e) => self.chest_editor.status_msg = format!("Error loading map: {}", e),
                }
                Task::none()
            }
            Message::ChestOpSelectChest(idx) => {
                self.chest_editor.selected_idx = Some(idx);
                if let Some((_, record)) = self.chest_editor.filtered_chests.get(idx) {
                    self.chest_editor.edit_name = record.name.clone();
                    self.chest_editor.edit_x = record.x_pos.to_string();
                    self.chest_editor.edit_y = record.y_pos.to_string();
                    self.chest_editor.edit_gold = record.gold_amount.to_string();
                    self.chest_editor.edit_item_count = record.item_count.to_string();
                    self.chest_editor.edit_item_id = record.item_id.to_string();
                    self.chest_editor.edit_item_type = (u8::from(record.item_type_id)).to_string();
                    self.chest_editor.edit_closed = record.closed.to_string();
                }
                Task::none()
            }
            Message::ChestOpFieldChanged(orig_idx, field, val) => {
                match field.as_str() {
                    "name" => self.chest_editor.edit_name = val.clone(),
                    "x" => self.chest_editor.edit_x = val.clone(),
                    "y" => self.chest_editor.edit_y = val.clone(),
                    "gold" => self.chest_editor.edit_gold = val.clone(),
                    "item_count" => self.chest_editor.edit_item_count = val.clone(),
                    "item_id" => self.chest_editor.edit_item_id = val.clone(),
                    "item_type" => self.chest_editor.edit_item_type = val.clone(),
                    "closed" => self.chest_editor.edit_closed = val.clone(),
                    _ => {}
                }
                if let Some(record) = self.chest_editor.all_records.get_mut(orig_idx) {
                    match field.as_str() {
                        "name" => record.name = val,
                        "x" => {
                            if let Ok(v) = val.parse() {
                                record.x_pos = v
                            }
                        }
                        "y" => {
                            if let Ok(v) = val.parse() {
                                record.y_pos = v
                            }
                        }
                        "gold" => {
                            if let Ok(v) = val.parse() {
                                record.gold_amount = v
                            }
                        }
                        "item_count" => {
                            if let Ok(v) = val.parse() {
                                record.item_count = v
                            }
                        }
                        "item_id" => {
                            if let Ok(v) = val.parse() {
                                record.item_id = v
                            }
                        }
                        "item_type" => {
                            if let Ok(v) = val.parse::<u8>() {
                                if let Some(t) = dispel_core::ItemTypeId::from_u8(v) {
                                    record.item_type_id = t;
                                }
                            }
                        }
                        "closed" => {
                            if let Ok(v) = val.parse() {
                                record.closed = v
                            }
                        }
                        _ => {}
                    }
                    self.refresh_chests();
                }
                Task::none()
            }
            Message::ChestOpSave => {
                if self.chest_editor.current_map_file.is_empty()
                    || self.chest_editor.all_records.is_empty()
                {
                    return Task::none();
                }
                self.chest_editor.is_loading = true;

                let path = PathBuf::from(&self.chest_editor.current_map_file);

                // Copy the original file with a timestamp (before file extension) as a backup
                if path.exists() {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    let ext = path.extension().unwrap_or_default().to_string_lossy();

                    let mut backup_path = path.clone();
                    backup_path.set_file_name(format!("{}_{}.{}", stem, timestamp, ext));

                    if let Err(e) = std::fs::copy(&path, &backup_path) {
                        return Task::perform(
                            async move { Err(format!("Failed to backup: {}", e)) },
                            |res| Message::ChestSaved(res),
                        );
                    }
                }

                let records = self.chest_editor.all_records.clone();
                Task::perform(
                    async move { dispel_core::ExtraRef::save_file(&records, &path) },
                    |res| Message::ChestSaved(res.map_err(|e| e.to_string())),
                )
            }
            Message::ChestSaved(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(_) => self.chest_editor.status_msg = "Map saved successfully.".into(),
                    Err(e) => self.chest_editor.status_msg = format!("Error saving map: {}", e),
                }
                Task::none()
            }
            Message::ChestOpAdd => Task::none(),
            Message::ChestOpDelete(_) => Task::none(),

            // ─── DB Viewer messages ─────────────────────────────────
            Message::ViewerDbPathChanged(v) => {
                self.viewer.db_path = v;
                Task::none()
            }
            Message::ViewerBrowseDb => browse_file("viewer_db"),
            Message::ViewerConnect => {
                self.viewer.is_loading = true;
                self.viewer.status_msg = "Connecting…".into();
                let path = self.viewer.db_path.clone();
                Task::perform(
                    async move { db::list_tables(&path) },
                    Message::ViewerTablesLoaded,
                )
            }
            Message::ViewerTablesLoaded(result) => {
                self.viewer.is_loading = false;
                match result {
                    Ok(tables) => {
                        self.viewer.status_msg = format!("Connected – {} tables", tables.len());
                        self.viewer.tables = tables;
                        self.viewer.active_table = None;
                        self.viewer.rows.clear();
                        self.viewer.columns.clear();
                    }
                    Err(e) => {
                        self.viewer.status_msg = format!("✖ {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerSelectTable(t) => {
                self.viewer.active_table = Some(t.clone());
                self.viewer.page = 0;
                self.viewer.search.clear();
                self.viewer.sort_col = None;
                self.viewer.pending_edits.clear();
                self.viewer.editing_cell = None;
                self.viewer.sql_mode = false;
                self.viewer.sql_query = format!("SELECT * FROM \"{}\"", t);
                self.fetch_viewer_data()
            }
            Message::ViewerDataLoaded(result) => {
                self.viewer.is_loading = false;
                match result {
                    Ok(qr) => {
                        self.viewer.columns = qr.columns;
                        self.viewer.rows = qr.rows;
                        self.viewer.total_rows = qr.total_rows;
                        let page_start = self.viewer.page * PAGE_SIZE + 1;
                        let page_end =
                            (page_start - 1 + self.viewer.rows.len()).max(page_start - 1);
                        self.viewer.status_msg = format!(
                            "Showing {}-{} of {} rows",
                            page_start, page_end, self.viewer.total_rows
                        );
                    }
                    Err(e) => {
                        self.viewer.status_msg = format!("✖ Query error: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerSearch(v) => {
                self.viewer.search = v;
                self.viewer.page = 0;
                self.fetch_viewer_data()
            }
            Message::ViewerSortColumn(idx) => {
                if self.viewer.sort_col == Some(idx) {
                    self.viewer.sort_dir = self.viewer.sort_dir.toggle();
                } else {
                    self.viewer.sort_col = Some(idx);
                    self.viewer.sort_dir = db::SortDir::Asc;
                }
                self.viewer.page = 0;
                self.fetch_viewer_data()
            }
            Message::ViewerNextPage => {
                let max_page = self.viewer.total_rows.saturating_sub(1) / PAGE_SIZE;
                if self.viewer.page < max_page {
                    self.viewer.page += 1;
                    return self.fetch_viewer_data();
                }
                Task::none()
            }
            Message::ViewerPrevPage => {
                if self.viewer.page > 0 {
                    self.viewer.page -= 1;
                    return self.fetch_viewer_data();
                }
                Task::none()
            }
            Message::ViewerCellClick(r, c) => {
                // Confirm previous edit if any
                if let Some((pr, pc)) = self.viewer.editing_cell {
                    if !self.viewer.edit_buffer.is_empty()
                        || self
                            .viewer
                            .rows
                            .get(pr)
                            .map(|row| row.get(pc).map(|v| v.as_str()))
                            .flatten()
                            != Some(&self.viewer.edit_buffer)
                    {
                        let original = self
                            .viewer
                            .rows
                            .get(pr)
                            .and_then(|row| row.get(pc))
                            .cloned()
                            .unwrap_or_default();
                        if self.viewer.edit_buffer != original {
                            self.viewer
                                .pending_edits
                                .insert((pr, pc), self.viewer.edit_buffer.clone());
                        }
                    }
                }
                let val = self
                    .viewer
                    .rows
                    .get(r)
                    .and_then(|row| row.get(c))
                    .cloned()
                    .unwrap_or_default();
                self.viewer.editing_cell = Some((r, c));
                self.viewer.edit_buffer = val;
                Task::none()
            }
            Message::ViewerCellEdit(v) => {
                self.viewer.edit_buffer = v;
                Task::none()
            }
            Message::ViewerCellConfirm => {
                if let Some((r, c)) = self.viewer.editing_cell {
                    let original = self
                        .viewer
                        .rows
                        .get(r)
                        .and_then(|row| row.get(c))
                        .cloned()
                        .unwrap_or_default();
                    if self.viewer.edit_buffer != original {
                        self.viewer
                            .pending_edits
                            .insert((r, c), self.viewer.edit_buffer.clone());
                    }
                }
                self.viewer.editing_cell = None;
                Task::none()
            }
            Message::ViewerCellCancel => {
                self.viewer.editing_cell = None;
                Task::none()
            }
            Message::ViewerCommit => {
                if self.viewer.pending_edits.is_empty() {
                    self.viewer.status_msg = "Nothing to commit.".into();
                    return Task::none();
                }
                let path = self.viewer.db_path.clone();
                let table = self.viewer.active_table.clone().unwrap_or_default();
                let cols = self.viewer.columns.clone();
                let rows = self.viewer.rows.clone();
                let edits = self.viewer.pending_edits.clone();
                self.viewer.is_loading = true;
                Task::perform(
                    async move { db::commit_edits(&path, &table, &cols, &rows, &edits) },
                    Message::ViewerCommitDone,
                )
            }
            Message::ViewerCommitDone(result) => {
                self.viewer.is_loading = false;
                match result {
                    Ok(n) => {
                        // Apply edits to local rows
                        for ((r, c), val) in &self.viewer.pending_edits {
                            if let Some(row) = self.viewer.rows.get_mut(*r) {
                                if let Some(cell) = row.get_mut(*c) {
                                    *cell = val.clone();
                                }
                            }
                        }
                        self.viewer.pending_edits.clear();
                        self.viewer.status_msg = format!("✔ Committed {} row(s)", n);
                    }
                    Err(e) => {
                        self.viewer.status_msg = format!("✖ Commit failed: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerToggleSql => {
                self.viewer.sql_mode = !self.viewer.sql_mode;
                Task::none()
            }
            Message::ViewerSqlChanged(v) => {
                self.viewer.sql_query = v;
                Task::none()
            }
            Message::ViewerRunSql => {
                self.viewer.page = 0;
                self.viewer.pending_edits.clear();
                self.viewer.editing_cell = None;
                self.fetch_viewer_data_sql()
            }
            Message::ViewerExportCsv => {
                let cols = self.viewer.columns.clone();
                let rows = self.viewer.rows.clone();
                Task::perform(
                    async move {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_file_name("export.csv")
                            .add_filter("CSV", &["csv"])
                            .save_file()
                            .await;
                        match handle {
                            Some(h) => {
                                let path = h.path().to_string_lossy().to_string();
                                db::export_csv(&path, &cols, &rows).map(|_| path)
                            }
                            None => Err("Cancelled".into()),
                        }
                    },
                    Message::ViewerCsvSaved,
                )
            }
            Message::ViewerCsvSaved(result) => {
                match result {
                    Ok(p) => self.viewer.status_msg = format!("✔ Exported to {}", p),
                    Err(e) => self.viewer.status_msg = format!("✖ Export: {}", e),
                }
                Task::none()
            }
            Message::ViewerRevertEdits => {
                self.viewer.pending_edits.clear();
                self.viewer.editing_cell = None;
                self.viewer.status_msg = "Reverted all pending edits.".into();
                Task::none()
            }
        }
    }

    /// Fetch data using the built table query (filters + sorting).
    fn fetch_viewer_data(&mut self) -> Task<Message> {
        let table = match &self.viewer.active_table {
            Some(t) => t.clone(),
            None => return Task::none(),
        };
        self.viewer.is_loading = true;

        // First get column info, then build query
        let path = self.viewer.db_path.clone();
        let search = self.viewer.search.clone();
        let sort_col = self.viewer.sort_col;
        let sort_dir = self.viewer.sort_dir;
        let page = self.viewer.page;

        Task::perform(
            async move {
                let cols = db::table_columns(&path, &table)?;
                let sql = db::build_table_query(&table, &cols, &search, sort_col, sort_dir);
                let mut result = db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE)?;
                // Merge proper column info
                result.columns = cols;
                Ok(result)
            },
            Message::ViewerDataLoaded,
        )
    }

    /// Fetch data using the custom SQL query.
    fn fetch_viewer_data_sql(&mut self) -> Task<Message> {
        self.viewer.is_loading = true;
        let path = self.viewer.db_path.clone();
        let sql = self.viewer.sql_query.clone();
        let page = self.viewer.page;

        Task::perform(
            async move { db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE) },
            Message::ViewerDataLoaded,
        )
    }

    fn build_internal_command(&self) -> Option<Box<dyn Command>> {
        let factory = CommandFactory::new();
        match self.active_tab {
            Tab::Map => {
                let op = self.map_op?;
                let subcommand = match op {
                    MapOp::Tiles => commands::map::MapSubcommand::Tiles {
                        input: self.map_input.clone(),
                        output: if self.map_output.is_empty() {
                            "out".to_string()
                        } else {
                            self.map_output.clone()
                        },
                    },
                    MapOp::Atlas => commands::map::MapSubcommand::Atlas {
                        input: self.map_input.clone(),
                        output: self.map_output.clone(),
                    },
                    MapOp::Render => commands::map::MapSubcommand::Render {
                        map: self.map_map_path.clone(),
                        btl: self.map_btl_path.clone(),
                        gtl: self.map_gtl_path.clone(),
                        output: self.map_output.clone(),
                        save_sprites: self.map_save_sprites,
                    },
                    MapOp::FromDb => commands::map::MapSubcommand::FromDb {
                        database: self.map_database.clone(),
                        map_id: self.map_map_id.clone(),
                        gtl_atlas: self.map_gtl_atlas.clone(),
                        btl_atlas: self.map_btl_atlas.clone(),
                        atlas_columns: self.map_atlas_columns.parse().unwrap_or(48),
                        output: self.map_output.clone(),
                        game_path: if self.map_game_path.is_empty() {
                            None
                        } else {
                            Some(self.map_game_path.clone())
                        },
                    },
                    MapOp::ToDb => commands::map::MapSubcommand::ToDb {
                        database: self.map_database.clone(),
                        map: self.map_map_path.clone(),
                    },
                    MapOp::Sprites => commands::map::MapSubcommand::Sprites {
                        input: self.map_input.clone(),
                        output: if self.map_output.is_empty() {
                            "out".to_string()
                        } else {
                            self.map_output.clone()
                        },
                    },
                };
                Some(Box::new(factory.create_map_command(subcommand)))
            }
            Tab::Ref => {
                let op = self.ref_op?;
                let input = self.ref_input.clone();
                let subcommand = match op {
                    RefOp::AllMaps => commands::ref_command::RefSubcommand::AllMaps { input },
                    RefOp::Map => commands::ref_command::RefSubcommand::Map { input },
                    RefOp::Extra => commands::ref_command::RefSubcommand::Extra { input },
                    RefOp::Event => commands::ref_command::RefSubcommand::Event { input },
                    RefOp::Monster => commands::ref_command::RefSubcommand::Monster { input },
                    RefOp::Npc => commands::ref_command::RefSubcommand::Npc { input },
                    RefOp::Wave => commands::ref_command::RefSubcommand::Wave { input },
                    RefOp::PartyRef => commands::ref_command::RefSubcommand::PartyRef { input },
                    RefOp::DrawItem => commands::ref_command::RefSubcommand::DrawItem { input },
                    RefOp::PartyPgp => commands::ref_command::RefSubcommand::PartyPgp { input },
                    RefOp::PartyDialog => {
                        commands::ref_command::RefSubcommand::PartyDialog { input }
                    }
                    RefOp::Dialog => commands::ref_command::RefSubcommand::Dialog { input },
                    RefOp::Weapons => commands::ref_command::RefSubcommand::Weapons { input },
                    RefOp::MultiMagic => commands::ref_command::RefSubcommand::MultiMagic { input },
                    RefOp::Store => commands::ref_command::RefSubcommand::Store { input },
                    RefOp::NpcRef => commands::ref_command::RefSubcommand::NpcRef { input },
                    RefOp::MonsterRef => commands::ref_command::RefSubcommand::MonsterRef { input },
                    RefOp::Monsters => commands::ref_command::RefSubcommand::Monsters { input },
                    RefOp::MiscItem => commands::ref_command::RefSubcommand::MiscItem { input },
                    RefOp::HealItems => commands::ref_command::RefSubcommand::HealItems { input },
                    RefOp::ExtraRef => commands::ref_command::RefSubcommand::ExtraRef { input },
                    RefOp::EventItems => commands::ref_command::RefSubcommand::EventItems { input },
                    RefOp::EditItems => commands::ref_command::RefSubcommand::EditItems { input },
                    RefOp::PartyLevel => commands::ref_command::RefSubcommand::PartyLevel { input },
                    RefOp::PartyIni => commands::ref_command::RefSubcommand::PartyIni { input },
                    RefOp::EventNpcRef => {
                        commands::ref_command::RefSubcommand::EventNpcRef { input }
                    }
                    RefOp::Magic => commands::ref_command::RefSubcommand::Magic { input },
                    RefOp::Quest => commands::ref_command::RefSubcommand::Quest { input },
                    RefOp::Message => commands::ref_command::RefSubcommand::Message { input },
                    RefOp::ChData => commands::ref_command::RefSubcommand::ChData { input },
                };
                Some(Box::new(factory.create_ref_command(subcommand)))
            }
            Tab::Database => {
                let op = self.db_op?;
                let subcommand = match op {
                    DbOp::Import => commands::database::DatabaseSubcommand::Import,
                    DbOp::DialogTexts => commands::database::DatabaseSubcommand::DialogTexts,
                    DbOp::Maps => commands::database::DatabaseSubcommand::Maps,
                    DbOp::Databases => commands::database::DatabaseSubcommand::Databases,
                    DbOp::Refs => commands::database::DatabaseSubcommand::Refs,
                    DbOp::Rest => commands::database::DatabaseSubcommand::Rest,
                };
                Some(Box::new(factory.create_database_command(subcommand)))
            }
            Tab::Sprite => {
                let mode = match self.sprite_mode {
                    Some(SpriteMode::Animation) => commands::sprite::SpriteMode::Animation,
                    _ => commands::sprite::SpriteMode::Sprite,
                };
                Some(Box::new(
                    factory.create_sprite_command(self.sprite_input.clone(), mode),
                ))
            }
            Tab::Sound => Some(Box::new(
                factory.create_sound_command(self.sound_input.clone(), self.sound_output.clone()),
            )),
            Tab::DbViewer | Tab::ChestEditor => None,
        }
    }

    // ─── View ───────────────────────────────────────────────────────────
    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = if self.active_tab == Tab::DbViewer {
            self.view_db_viewer()
        } else if self.active_tab == Tab::ChestEditor {
            self.view_chest_editor_tab()
        } else {
            let tab_content = self.view_tab_content();
            let log_panel = self.view_log();
            column![tab_content, horizontal_rule(1), log_panel]
                .spacing(0)
                .width(Fill)
                .height(Fill)
                .into()
        };

        let layout = row![sidebar, content].height(Fill).width(Fill);
        container(layout)
            .width(Fill)
            .height(Fill)
            .style(style::root_container)
            .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let title = text("Dispel Extractor").size(18).font(Font::MONOSPACE);
        let tabs: Vec<Element<Message>> = Tab::ALL
            .iter()
            .map(|tab| {
                let is_active = *tab == self.active_tab;
                let btn = button(text(tab.label()).size(14))
                    .width(Fill)
                    .padding([10, 16])
                    .on_press(Message::TabSelected(*tab));
                if is_active {
                    btn.style(style::active_tab_button)
                } else {
                    btn.style(style::tab_button)
                }
                .into()
            })
            .collect();
        let sidebar_content = column![
            vertical_space().height(12),
            container(title).padding([0, 16]),
            vertical_space().height(16),
            column(tabs).spacing(2).padding([0, 8]),
            vertical_space().height(Length::Fill),
            vertical_space().height(8),
        ]
        .spacing(0)
        .width(220);
        container(sidebar_content)
            .height(Fill)
            .style(style::sidebar_container)
            .into()
    }

    fn view_tab_content(&self) -> Element<'_, Message> {
        let inner = match self.active_tab {
            Tab::Map => self.view_map_tab(),
            Tab::Ref => self.view_ref_tab(),
            Tab::Database => self.view_database_tab(),
            Tab::Sprite => self.view_sprite_tab(),
            Tab::Sound => self.view_sound_tab(),
            Tab::DbViewer => text("").into(),
            Tab::ChestEditor => self.view_chest_editor_tab(),
        };
        let run_btn = if self.is_running {
            button(text("⏳ Running…").size(14))
                .padding([10, 28])
                .style(style::run_button_disabled)
        } else {
            button(text("▶  Run Command").size(14))
                .padding([10, 28])
                .on_press(Message::Run)
                .style(style::run_button)
        };
        let header = text(match self.active_tab {
            Tab::Map => "Map Operations",
            Tab::Ref => "Reference Data Extraction",
            Tab::Database => "Database Import Pipeline",
            Tab::Sprite => "Sprite / Animation Extraction",
            Tab::Sound => "Audio Conversion (SNF → WAV)",
            _ => "",
        })
        .size(22);
        let subtitle = text(match self.active_tab {
            Tab::Map => "Extract tiles, render maps, and manage map assets",
            Tab::Ref => "Read game DB/INI/REF files and output as JSON",
            Tab::Database => "Populate a local SQLite database from game fixtures",
            Tab::Sprite => "Parse .SPR / .SPX files to extract frames or sequences",
            Tab::Sound => "Convert .SNF game audio to standard .WAV format",
            _ => "",
        })
        .size(13)
        .style(style::subtle_text);
        let content = column![
            header,
            subtitle,
            vertical_space().height(16),
            inner,
            vertical_space().height(16),
            row![horizontal_space(), run_btn].width(Fill)
        ]
        .spacing(4)
        .padding(24)
        .width(Fill);
        scrollable(content).height(Length::FillPortion(3)).into()
    }

    fn view_map_tab(&self) -> Element<'_, Message> {
        let op_buttons: Vec<Element<Message>> = MapOp::ALL
            .iter()
            .map(|op| {
                let is_active = self.map_op == Some(*op);
                let btn = button(text(op.label()).size(12))
                    .padding([6, 12])
                    .on_press(Message::MapOpSelected(*op));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        let fields: Element<Message> = match self.map_op {
            Some(MapOp::Tiles) | Some(MapOp::Sprites) => column![
                labeled_file_row(
                    "Input:",
                    &self.map_input,
                    Message::MapInputChanged,
                    Message::BrowseMapInput
                ),
                labeled_input("Output dir:", &self.map_output, Message::MapOutputChanged),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::Atlas) => column![
                labeled_file_row(
                    "Input:",
                    &self.map_input,
                    Message::MapInputChanged,
                    Message::BrowseMapInput
                ),
                labeled_input("Output PNG:", &self.map_output, Message::MapOutputChanged),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::Render) => column![
                labeled_file_row(
                    "MAP file:",
                    &self.map_map_path,
                    Message::MapMapPathChanged,
                    Message::BrowseMapMapPath
                ),
                labeled_file_row(
                    "BTL file:",
                    &self.map_btl_path,
                    Message::MapBtlPathChanged,
                    Message::BrowseMapBtlPath
                ),
                labeled_file_row(
                    "GTL file:",
                    &self.map_gtl_path,
                    Message::MapGtlPathChanged,
                    Message::BrowseMapGtlPath
                ),
                labeled_input("Output PNG:", &self.map_output, Message::MapOutputChanged),
                toggler(self.map_save_sprites)
                    .label("Save sprites")
                    .on_toggle(Message::MapSaveSpritesToggled)
                    .size(18)
                    .spacing(8),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::FromDb) => column![
                labeled_input("Database:", &self.map_database, Message::MapDatabaseChanged),
                labeled_input("Map ID:", &self.map_map_id, Message::MapMapIdChanged),
                labeled_file_row(
                    "GTL Atlas:",
                    &self.map_gtl_atlas,
                    Message::MapGtlAtlasChanged,
                    Message::BrowseMapGtlAtlas
                ),
                labeled_file_row(
                    "BTL Atlas:",
                    &self.map_btl_atlas,
                    Message::MapBtlAtlasChanged,
                    Message::BrowseMapBtlAtlas
                ),
                labeled_input(
                    "Columns:",
                    &self.map_atlas_columns,
                    Message::MapAtlasColumnsChanged
                ),
                labeled_input("Output:", &self.map_output, Message::MapOutputChanged),
                labeled_file_row(
                    "Game Path:",
                    &self.map_game_path,
                    Message::MapGamePathChanged,
                    Message::BrowseMapGamePath
                ),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::ToDb) => column![
                labeled_input("Database:", &self.map_database, Message::MapDatabaseChanged),
                labeled_file_row(
                    "MAP file:",
                    &self.map_map_path,
                    Message::MapMapPathChanged,
                    Message::BrowseMapMapPath
                ),
            ]
            .spacing(10)
            .into(),
            None => text("Select an operation above.").into(),
        };
        column![
            row(op_buttons).spacing(6).wrap(),
            vertical_space().height(12),
            fields
        ]
        .spacing(4)
        .into()
    }

    fn view_ref_tab(&self) -> Element<'_, Message> {
        let picker = pick_list(RefOp::ALL, self.ref_op, Message::RefOpSelected)
            .placeholder("Select…")
            .padding(8);
        column![
            row![text("Ref type:").size(13).width(140), picker]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            labeled_file_row(
                "Input file:",
                &self.ref_input,
                Message::RefInputChanged,
                Message::BrowseRefInput
            ),
        ]
        .spacing(12)
        .into()
    }

    fn view_database_tab(&self) -> Element<'_, Message> {
        let op_buttons: Vec<Element<Message>> = DbOp::ALL
            .iter()
            .map(|op| {
                let is_active = self.db_op == Some(*op);
                let btn = button(text(op.label()).size(12))
                    .padding([6, 14])
                    .on_press(Message::DbOpSelected(*op));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        let desc = match self.db_op {
            Some(DbOp::Import) => {
                "Imports everything: maps → refs → rest → dialog-texts → databases"
            }
            Some(DbOp::DialogTexts) => "Imports .dlg and .pgp dialogue files",
            Some(DbOp::Maps) => "Imports .map files + AllMap.ini + Map.ini",
            Some(DbOp::Databases) => {
                "Imports .db files (weapons, stores, monsters, items, spells, quests, messages)"
            }
            Some(DbOp::Refs) => "Imports INI config files (Extra, Event, Monster, Npc, Wave)",
            Some(DbOp::Rest) => "Imports .ref and .pgp reference files",
            None => "Select an operation above.",
        };
        column![
            row(op_buttons).spacing(6).wrap(),
            vertical_space().height(12),
            container(text(desc).size(13))
                .padding(12)
                .style(style::info_card)
        ]
        .spacing(8)
        .into()
    }

    fn view_sprite_tab(&self) -> Element<'_, Message> {
        let mp = pick_list(
            SpriteMode::ALL,
            self.sprite_mode,
            Message::SpriteModeSelected,
        )
        .padding(8);
        column![
            labeled_file_row(
                "Input (.SPR/.SPX):",
                &self.sprite_input,
                Message::SpriteInputChanged,
                Message::BrowseSpriteInput
            ),
            row![text("Mode:").size(13).width(140), mp]
                .spacing(8)
                .align_y(iced::Alignment::Center),
        ]
        .spacing(12)
        .into()
    }

    fn view_sound_tab(&self) -> Element<'_, Message> {
        column![
            labeled_file_row(
                "Input .SNF:",
                &self.sound_input,
                Message::SoundInputChanged,
                Message::BrowseSoundInput
            ),
            labeled_file_row(
                "Output .WAV:",
                &self.sound_output,
                Message::SoundOutputChanged,
                Message::BrowseSoundOutput
            ),
        ]
        .spacing(12)
        .into()
    }

    fn view_log(&self) -> Element<'_, Message> {
        let clear_btn = button(text("Clear").size(11))
            .padding([4, 12])
            .on_press(Message::ClearLog)
            .style(style::chip);
        let header = row![text("Output Log").size(13), horizontal_space(), clear_btn]
            .align_y(iced::Alignment::Center)
            .padding([8, 12]);
        let log_text = text(&self.log).size(12).font(Font::MONOSPACE);
        let log_scroll = scrollable(container(log_text).padding([4, 12]).width(Fill)).height(Fill);
        container(column![header, log_scroll].spacing(0))
            .height(Length::FillPortion(2))
            .width(Fill)
            .style(style::log_container)
            .into()
    }

    // ─── DB Viewer (full-tab view) ──────────────────────────────────
    fn view_db_viewer(&self) -> Element<'_, Message> {
        let v = &self.viewer;

        // ── Connection toolbar ──
        let db_input = text_input("database.sqlite", &v.db_path)
            .on_input(Message::ViewerDbPathChanged)
            .padding(8)
            .size(13);
        let browse_btn = button(text("…").size(12))
            .padding([6, 10])
            .on_press(Message::ViewerBrowseDb)
            .style(style::browse_button);
        let connect_btn = button(text("Connect").size(12))
            .padding([6, 14])
            .on_press(Message::ViewerConnect)
            .style(style::run_button);
        let conn_row = container(
            row![
                text("Database:").size(13),
                db_input,
                browse_btn,
                connect_btn
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(10),
        )
        .width(Fill)
        .style(style::toolbar_container);

        // ── Table selector chips ──
        let table_chips: Vec<Element<Message>> = v
            .tables
            .iter()
            .map(|t| {
                let is_active = v.active_table.as_deref() == Some(t.as_str());
                let btn = button(text(t).size(11))
                    .padding([4, 10])
                    .on_press(Message::ViewerSelectTable(t.clone()));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        let table_row = if table_chips.is_empty() {
            container(
                text("Connect to a database to see tables.")
                    .size(12)
                    .style(style::subtle_text),
            )
            .padding(10)
        } else {
            container(scrollable(row(table_chips).spacing(4).padding(6).wrap()).height(60))
                .padding([4, 10])
        };

        // ── Action toolbar ──
        let search_input = text_input("🔍 Search all columns…", &v.search)
            .on_input(Message::ViewerSearch)
            .padding(8)
            .size(12)
            .width(250);
        let sql_toggle = button(text(if v.sql_mode { "Hide SQL" } else { "SQL Editor" }).size(11))
            .padding([5, 10])
            .on_press(Message::ViewerToggleSql)
            .style(style::chip);
        let export_btn = button(text("📥 Export CSV").size(11))
            .padding([5, 10])
            .on_press(Message::ViewerExportCsv)
            .style(style::chip);

        let edit_count = v.pending_edits.len();
        let commit_btn = if edit_count > 0 {
            button(text(format!("💾 Commit ({edit_count})")).size(11))
                .padding([5, 10])
                .on_press(Message::ViewerCommit)
                .style(style::commit_button)
        } else {
            button(text("💾 Commit").size(11))
                .padding([5, 10])
                .style(style::run_button_disabled)
        };
        let revert_btn = if edit_count > 0 {
            button(text("↩ Revert").size(11))
                .padding([5, 10])
                .on_press(Message::ViewerRevertEdits)
                .style(style::chip)
        } else {
            button(text("↩ Revert").size(11))
                .padding([5, 10])
                .style(style::run_button_disabled)
        };

        let action_row = container(
            row![
                search_input,
                horizontal_space(),
                sql_toggle,
                export_btn,
                revert_btn,
                commit_btn
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .padding(8),
        )
        .width(Fill)
        .style(style::toolbar_container);

        // ── SQL editor (collapsible) ──
        let sql_area: Element<Message> = if v.sql_mode {
            let sql_input = text_input("SELECT * FROM ...", &v.sql_query)
                .on_input(Message::ViewerSqlChanged)
                .padding(10)
                .size(13)
                .font(Font::MONOSPACE);
            let run_btn = button(text("▶ Run").size(12))
                .padding([6, 14])
                .on_press(Message::ViewerRunSql)
                .style(style::run_button);
            container(
                row![sql_input, run_btn]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                    .padding(8),
            )
            .width(Fill)
            .style(style::sql_editor_container)
            .into()
        } else {
            vertical_space().height(0).into()
        };

        // ── Data grid ──
        let grid = self.view_grid();

        // ── Pagination ──
        let max_page = if v.total_rows == 0 {
            0
        } else {
            v.total_rows.saturating_sub(1) / PAGE_SIZE
        };
        let prev_btn = if v.page > 0 {
            button(text("◀ Prev").size(11))
                .padding([4, 10])
                .on_press(Message::ViewerPrevPage)
                .style(style::chip)
        } else {
            button(text("◀ Prev").size(11))
                .padding([4, 10])
                .style(style::run_button_disabled)
        };
        let next_btn = if v.page < max_page {
            button(text("Next ▶").size(11))
                .padding([4, 10])
                .on_press(Message::ViewerNextPage)
                .style(style::chip)
        } else {
            button(text("Next ▶").size(11))
                .padding([4, 10])
                .style(style::run_button_disabled)
        };
        let page_info = text(format!("Page {} / {}", v.page + 1, max_page + 1)).size(11);

        let status_row = container(
            row![
                text(&v.status_msg).size(11).style(style::subtle_text),
                horizontal_space(),
                prev_btn,
                page_info,
                next_btn
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding([6, 12]),
        )
        .width(Fill)
        .style(style::status_bar);

        column![conn_row, table_row, action_row, sql_area, grid, status_row]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn view_grid(&self) -> Element<'_, Message> {
        let v = &self.viewer;
        if v.columns.is_empty() {
            return container(
                text("Select a table to view its data.")
                    .size(14)
                    .style(style::subtle_text),
            )
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }

        // ── Header row ──
        let header_cells: Vec<Element<Message>> = v
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let sort_indicator = if v.sort_col == Some(i) {
                    v.sort_dir.arrow()
                } else {
                    ""
                };
                let label = format!("{}{}", col.name, sort_indicator);
                let pk_marker = if col.is_pk { " 🔑" } else { "" };
                button(
                    text(format!("{label}{pk_marker}"))
                        .size(11)
                        .font(Font::MONOSPACE),
                )
                .width(150)
                .padding([8, 6])
                .on_press(Message::ViewerSortColumn(i))
                .style(style::sort_button)
                .into()
            })
            .collect();
        // Removed .width(Fill) to prevent iced panic in horizontal scrollable
        let header = container(row(header_cells).spacing(0)).style(style::grid_header_cell);

        // ── Data rows ──
        let data_rows: Vec<Element<Message>> = v
            .rows
            .iter()
            .enumerate()
            .map(|(ri, row_data)| {
                let cells: Vec<Element<Message>> = row_data
                    .iter()
                    .enumerate()
                    .map(|(ci, cell_val)| {
                        let is_editing = v.editing_cell == Some((ri, ci));
                        let is_dirty = v.pending_edits.contains_key(&(ri, ci));
                        let display_val = v.pending_edits.get(&(ri, ci)).unwrap_or(cell_val);

                        let cell_style = if is_dirty {
                            style::grid_cell_dirty
                        } else if ri % 2 == 0 {
                            style::grid_cell
                        } else {
                            style::grid_cell_even
                        };

                        let inner: Element<Message> = if is_editing {
                            text_input("", &v.edit_buffer)
                                .on_input(Message::ViewerCellEdit)
                                .on_submit(Message::ViewerCellConfirm)
                                .padding(4)
                                .size(11)
                                .font(Font::MONOSPACE)
                                .into()
                        } else {
                            button(text(display_val).size(11).font(Font::MONOSPACE))
                                .width(Fill)
                                .padding([6, 4])
                                .on_press(Message::ViewerCellClick(ri, ci))
                                .style(style::sort_button)
                                .into()
                        };

                        container(inner).width(150).style(cell_style).into()
                    })
                    .collect();
                row(cells).spacing(0).into()
            })
            .collect();

        // Removed .width(Fill) below
        let grid_content = column![header, column(data_rows).spacing(0)].spacing(0);

        scrollable(grid_content)
            .direction(iced::widget::scrollable::Direction::Both {
                vertical: Default::default(),
                horizontal: Default::default(),
            })
            .height(Fill)
            .width(Fill)
            .into()
    }

    fn view_chest_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.chest_editor;

        let game_path_row = row![
            text("Game Path:").size(14),
            text(format!("{}", editor.game_path)).size(12),
            button(text("Browse..."))
                .on_press(Message::ChestOpBrowseGamePath)
                .style(style::browse_button),
            button(text("Load Catalog"))
                .on_press(Message::ChestOpLoadCatalog)
                .style(style::run_button),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let map_file_row = row![
            text("Map File:").size(14),
            text(format!("{}", editor.current_map_file)).size(12),
            button(text("Browse..."))
                .on_press(Message::ChestOpBrowseMapFile)
                .style(style::browse_button),
            button(text("Load Map"))
                .on_press(Message::ChestOpSelectMap)
                .style(style::run_button),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let status_row = container(
            row![
                text(&editor.status_msg).size(13).style(style::subtle_text),
                horizontal_space(),
                if editor.is_loading {
                    Element::from(text("Loading...").size(13))
                } else {
                    Element::from(text(""))
                },
                horizontal_space().width(20),
                button(text("Save Map Changes"))
                    .on_press(Message::ChestOpSave)
                    .style(style::commit_button),
            ]
            .padding([10, 20])
            .align_y(iced::Alignment::Center),
        )
        .width(Fill)
        .style(style::status_bar);

        let map_list: Vec<Element<Message>> = editor
            .map_files
            .iter()
            .map(|path| {
                let is_selected = editor.current_map_file == path.to_string_lossy();
                let btn =
                    button(text(path.file_name().unwrap_or_default().to_string_lossy()).size(12))
                        .width(Fill)
                        .on_press(Message::ChestOpSelectMapFromFile(path.clone()));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let chest_list: Vec<Element<Message>> = editor
            .filtered_chests
            .iter()
            .enumerate()
            .map(|(idx, (_, record))| {
                let is_selected = editor.selected_idx == Some(idx);
                let item_name = editor
                    .catalog
                    .as_ref()
                    .and_then(|c| c.get_item_name(record.item_type_id, record.item_id))
                    .unwrap_or_else(|| format!("{:?}_{}", record.item_type_id, record.item_id));

                let label = format!(
                    "Chest [{}] x:{} y:{}\n  {} (x{})\n  {} gold",
                    record.id,
                    record.x_pos,
                    record.y_pos,
                    item_name,
                    record.item_count,
                    record.gold_amount
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::ChestOpSelectChest(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Chest Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, record)) = editor.filtered_chests.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::ChestOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input("X Pos:", &editor.edit_x, move |v| {
                    Message::ChestOpFieldChanged(orig, "x".into(), v)
                }));
                detail_content.push(labeled_input("Y Pos:", &editor.edit_y, move |v| {
                    Message::ChestOpFieldChanged(orig, "y".into(), v)
                }));
                detail_content.push(labeled_input("Gold:", &editor.edit_gold, move |v| {
                    Message::ChestOpFieldChanged(orig, "gold".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Item Count:",
                    &editor.edit_item_count,
                    move |v| Message::ChestOpFieldChanged(orig, "item_count".into(), v),
                ));
                detail_content.push(labeled_input("Item ID:", &editor.edit_item_id, move |v| {
                    Message::ChestOpFieldChanged(orig, "item_id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Item Type:",
                    &editor.edit_item_type,
                    move |v| Message::ChestOpFieldChanged(orig, "item_type".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Closed (0=open, 1=closed):",
                    &editor.edit_closed,
                    move |v| Message::ChestOpFieldChanged(orig, "closed".into(), v),
                ));

                let item_name = editor
                    .catalog
                    .as_ref()
                    .and_then(|c| c.get_item_name(record.item_type_id, record.item_id))
                    .unwrap_or_default();
                if !item_name.is_empty() {
                    detail_content.push(
                        text(format!("Resolved Item: {}", item_name))
                            .size(12)
                            .style(style::subtle_text)
                            .into(),
                    );
                }
            }
        } else {
            detail_content.push(
                text("No chest selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_panel = container(scrollable(column(detail_content).spacing(8)).height(Fill))
            .padding(16)
            .width(250)
            .style(style::info_card);

        let list_header = row![
            text("Chests").size(14),
            horizontal_space(),
            text(format!("{} found", editor.filtered_chests.len()))
                .size(12)
                .style(style::subtle_text)
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let main_content = row![
            column![
                container(
                    row![
                        text("Maps").size(14),
                        horizontal_space(),
                        button(text("Scan"))
                            .on_press(Message::ChestOpScanMaps)
                            .style(style::chip)
                    ]
                    .padding(10)
                    .align_y(iced::Alignment::Center)
                )
                .style(style::grid_header_cell),
                scrollable(column(map_list)).height(Fill),
            ]
            .width(180),
            column![
                container(list_header).style(style::grid_header_cell),
                scrollable(column(chest_list)).height(Fill),
            ]
            .width(Fill),
            detail_panel,
        ]
        .spacing(0)
        .height(Fill);

        column![
            container(
                row![game_path_row, horizontal_space(), map_file_row]
                    .padding(10)
                    .align_y(iced::Alignment::Center)
            )
            .style(style::toolbar_container),
            horizontal_rule(1),
            main_content,
            status_row,
        ]
        .spacing(0)
        .into()
    }
}

// ─── Helpers (unchanged) ────────────────────────────────────────────────────
fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).size(13).width(140),
        text_input("", value)
            .on_input(on_change)
            .padding(8)
            .size(13)
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn labeled_file_row<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
    browse_msg: Message,
) -> Element<'a, Message> {
    row![
        text(label).size(13).width(140),
        text_input("", value)
            .on_input(on_change)
            .padding(8)
            .size(13),
        button(text("…").size(12))
            .padding([6, 10])
            .on_press(browse_msg)
            .style(style::browse_button),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn browse_file(field: &str) -> Task<Message> {
    let field = field.to_string();
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .pick_file()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |path| Message::FileSelected {
            field: field.clone(),
            path,
        },
    )
}

fn browse_folder(field: &str) -> Task<Message> {
    let field = field.to_string();
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |path| Message::FileSelected {
            field: field.clone(),
            path,
        },
    )
}

async fn run_command(exe: String, args: Vec<String>) -> Result<String, String> {
    use tokio::process::Command;
    let output = Command::new(&exe)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn '{}': {}", exe, e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        result.push_str(&stderr);
    }
    if output.status.success() {
        Ok(result)
    } else {
        Err(format!(
            "Exit code {}.\n{}",
            output.status.code().unwrap_or(-1),
            result
        ))
    }
}
