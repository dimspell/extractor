use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable, text,
    text_input, toggler, vertical_space,
};
use iced::{color, Element, Fill, Font, Length, Task, Theme};
use std::collections::HashMap;
use std::path::PathBuf;

mod db;
mod style;

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
}

impl Tab {
    const ALL: &'static [Tab] = &[
        Tab::Map,
        Tab::Ref,
        Tab::Database,
        Tab::Sprite,
        Tab::Sound,
        Tab::DbViewer,
    ];
    fn label(&self) -> &str {
        match self {
            Tab::Map => "🗺  Map",
            Tab::Ref => "📋  Ref",
            Tab::Database => "🗄  Database",
            Tab::Sprite => "🎨  Sprite",
            Tab::Sound => "🔊  Sound",
            Tab::DbViewer => "📊  DB Viewer",
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
    viewer: DbViewerState,
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
                viewer: DbViewerState::default(),
            },
            Task::none(),
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
                let args = self.build_args();
                if args.is_empty() {
                    self.log.push_str("⚠ No command configured.\n");
                    return Task::none();
                }
                self.log
                    .push_str(&format!("▸ {} {}\n", self.extractor_path, args.join(" ")));
                self.is_running = true;
                let exe = self.extractor_path.clone();
                Task::perform(
                    async move { run_command(exe, args).await },
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

    fn build_args(&self) -> Vec<String> {
        match self.active_tab {
            Tab::Map => self.build_map_args(),
            Tab::Ref => self.build_ref_args(),
            Tab::Database => self.build_db_args(),
            Tab::Sprite => self.build_sprite_args(),
            Tab::Sound => self.build_sound_args(),
            Tab::DbViewer => vec![],
        }
    }
    fn build_map_args(&self) -> Vec<String> {
        let op = match self.map_op {
            Some(op) => op,
            None => return vec![],
        };
        let mut a = vec!["map".into()];
        match op {
            MapOp::Tiles => {
                a.extend(["tiles".into(), self.map_input.clone()]);
                if !self.map_output.is_empty() {
                    a.extend(["--output".into(), self.map_output.clone()]);
                }
            }
            MapOp::Atlas => {
                a.extend([
                    "atlas".into(),
                    self.map_input.clone(),
                    self.map_output.clone(),
                ]);
            }
            MapOp::Render => {
                a.extend([
                    "render".into(),
                    "--map".into(),
                    self.map_map_path.clone(),
                    "--btl".into(),
                    self.map_btl_path.clone(),
                    "--gtl".into(),
                    self.map_gtl_path.clone(),
                    "--output".into(),
                    self.map_output.clone(),
                ]);
                if self.map_save_sprites {
                    a.push("--save-sprites".into());
                }
            }
            MapOp::FromDb => {
                a.extend([
                    "from-db".into(),
                    "--database".into(),
                    self.map_database.clone(),
                    "--map-id".into(),
                    self.map_map_id.clone(),
                    "--gtl-atlas".into(),
                    self.map_gtl_atlas.clone(),
                    "--btl-atlas".into(),
                    self.map_btl_atlas.clone(),
                    "--atlas-columns".into(),
                    self.map_atlas_columns.clone(),
                    "--output".into(),
                    self.map_output.clone(),
                ]);
                if !self.map_game_path.is_empty() {
                    a.extend(["--game-path".into(), self.map_game_path.clone()]);
                }
            }
            MapOp::ToDb => {
                a.extend([
                    "to-db".into(),
                    "--database".into(),
                    self.map_database.clone(),
                    "--map".into(),
                    self.map_map_path.clone(),
                ]);
            }
            MapOp::Sprites => {
                a.extend(["sprites".into(), self.map_input.clone()]);
                if !self.map_output.is_empty() {
                    a.extend(["--output".into(), self.map_output.clone()]);
                }
            }
        }
        a
    }
    fn build_ref_args(&self) -> Vec<String> {
        match self.ref_op {
            Some(op) => vec!["ref".into(), op.cli_name().into(), self.ref_input.clone()],
            None => vec![],
        }
    }
    fn build_db_args(&self) -> Vec<String> {
        match self.db_op {
            Some(op) => vec!["database".into(), op.cli_name().into()],
            None => vec![],
        }
    }
    fn build_sprite_args(&self) -> Vec<String> {
        let m = match self.sprite_mode {
            Some(SpriteMode::Animation) => "animation",
            _ => "sprite",
        };
        vec![
            "sprite".into(),
            self.sprite_input.clone(),
            format!("--mode={m}"),
        ]
    }
    fn build_sound_args(&self) -> Vec<String> {
        vec![
            "sound".into(),
            self.sound_input.clone(),
            self.sound_output.clone(),
        ]
    }

    // ─── View ───────────────────────────────────────────────────────────
    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = if self.active_tab == Tab::DbViewer {
            self.view_db_viewer()
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
        let ext_label = text("Extractor binary:").size(11);
        let ext_input = text_input("path/to/dispel-extractor", &self.extractor_path)
            .on_input(Message::ExtractorPathChanged)
            .size(11)
            .padding(6);
        let ext_browse = button(text("…").size(11))
            .padding([4, 8])
            .on_press(Message::BrowseExtractorPath)
            .style(style::browse_button);
        let sidebar_content = column![
            vertical_space().height(12),
            container(title).padding([0, 16]),
            vertical_space().height(16),
            column(tabs).spacing(2).padding([0, 8]),
            vertical_space().height(Length::Fill),
            container(column![ext_label, row![ext_input, ext_browse].spacing(4)].spacing(4))
                .padding([8, 12]),
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
