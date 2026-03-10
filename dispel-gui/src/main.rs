use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row,
    scrollable, text, text_input, toggler, vertical_space,
};
use iced::{color, Element, Fill, Font, Length, Task, Theme};
use std::path::PathBuf;

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
        .window_size((980.0, 760.0))
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
}

impl Tab {
    const ALL: &'static [Tab] = &[Tab::Map, Tab::Ref, Tab::Database, Tab::Sprite, Tab::Sound];
    fn label(&self) -> &str {
        match self {
            Tab::Map => "🗺  Map",
            Tab::Ref => "📋  Ref",
            Tab::Database => "🗄  Database",
            Tab::Sprite => "🎨  Sprite",
            Tab::Sound => "🔊  Sound",
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
    BrowseMapOutput,
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
            Message::BrowseMapOutput => browse_file("map_output"),
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
                        "map_output" => self.map_output = s,
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
                let cmd_str = format!("{} {}\n", self.extractor_path, args.join(" "));
                self.log.push_str(&format!("▸ {}", cmd_str));
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
        }
    }

    fn build_args(&self) -> Vec<String> {
        match self.active_tab {
            Tab::Map => self.build_map_args(),
            Tab::Ref => self.build_ref_args(),
            Tab::Database => self.build_db_args(),
            Tab::Sprite => self.build_sprite_args(),
            Tab::Sound => self.build_sound_args(),
        }
    }

    fn build_map_args(&self) -> Vec<String> {
        let op = match self.map_op {
            Some(op) => op,
            None => return vec![],
        };
        let mut args = vec!["map".to_string()];
        match op {
            MapOp::Tiles => {
                args.push("tiles".into());
                args.push(self.map_input.clone());
                if !self.map_output.is_empty() {
                    args.push("--output".into());
                    args.push(self.map_output.clone());
                }
            }
            MapOp::Atlas => {
                args.push("atlas".into());
                args.push(self.map_input.clone());
                args.push(self.map_output.clone());
            }
            MapOp::Render => {
                args.push("render".into());
                args.push("--map".into());
                args.push(self.map_map_path.clone());
                args.push("--btl".into());
                args.push(self.map_btl_path.clone());
                args.push("--gtl".into());
                args.push(self.map_gtl_path.clone());
                args.push("--output".into());
                args.push(self.map_output.clone());
                if self.map_save_sprites {
                    args.push("--save-sprites".into());
                }
            }
            MapOp::FromDb => {
                args.push("from-db".into());
                args.push("--database".into());
                args.push(self.map_database.clone());
                args.push("--map-id".into());
                args.push(self.map_map_id.clone());
                args.push("--gtl-atlas".into());
                args.push(self.map_gtl_atlas.clone());
                args.push("--btl-atlas".into());
                args.push(self.map_btl_atlas.clone());
                args.push("--atlas-columns".into());
                args.push(self.map_atlas_columns.clone());
                args.push("--output".into());
                args.push(self.map_output.clone());
                if !self.map_game_path.is_empty() {
                    args.push("--game-path".into());
                    args.push(self.map_game_path.clone());
                }
            }
            MapOp::ToDb => {
                args.push("to-db".into());
                args.push("--database".into());
                args.push(self.map_database.clone());
                args.push("--map".into());
                args.push(self.map_map_path.clone());
            }
            MapOp::Sprites => {
                args.push("sprites".into());
                args.push(self.map_input.clone());
                if !self.map_output.is_empty() {
                    args.push("--output".into());
                    args.push(self.map_output.clone());
                }
            }
        }
        args
    }

    fn build_ref_args(&self) -> Vec<String> {
        let op = match self.ref_op {
            Some(op) => op,
            None => return vec![],
        };
        vec![
            "ref".to_string(),
            op.cli_name().to_string(),
            self.ref_input.clone(),
        ]
    }

    fn build_db_args(&self) -> Vec<String> {
        let op = match self.db_op {
            Some(op) => op,
            None => return vec![],
        };
        vec!["database".to_string(), op.cli_name().to_string()]
    }

    fn build_sprite_args(&self) -> Vec<String> {
        let mode_str = match self.sprite_mode {
            Some(SpriteMode::Animation) => "animation",
            _ => "sprite",
        };
        vec![
            "sprite".to_string(),
            self.sprite_input.clone(),
            format!("--mode={}", mode_str),
        ]
    }

    fn build_sound_args(&self) -> Vec<String> {
        vec![
            "sound".to_string(),
            self.sound_input.clone(),
            self.sound_output.clone(),
        ]
    }

    // ─── View ───────────────────────────────────────────────────────────
    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = self.view_tab_content();
        let log_panel = self.view_log();

        let main_area = column![content, horizontal_rule(1), log_panel,]
            .spacing(0)
            .width(Fill)
            .height(Fill);

        let layout = row![sidebar, main_area].height(Fill).width(Fill);

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

                let btn = if is_active {
                    btn.style(style::active_tab_button)
                } else {
                    btn.style(style::tab_button)
                };

                btn.into()
            })
            .collect();

        let extractor_label = text("Extractor binary:").size(11);
        let extractor_input = text_input("path/to/dispel-extractor", &self.extractor_path)
            .on_input(Message::ExtractorPathChanged)
            .size(11)
            .padding(6);
        let browse_ext_btn = button(text("…").size(11))
            .padding([4, 8])
            .on_press(Message::BrowseExtractorPath)
            .style(style::browse_button);

        let extractor_row = row![extractor_input, browse_ext_btn].spacing(4);

        let sidebar_content = column![
            vertical_space().height(12),
            container(title).padding([0, 16]),
            vertical_space().height(16),
            column(tabs).spacing(2).padding([0, 8]),
            vertical_space().height(Length::Fill),
            container(column![extractor_label, extractor_row].spacing(4)).padding([8, 12]),
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
        })
        .size(22);

        let subtitle = text(match self.active_tab {
            Tab::Map => "Extract tiles, render maps, and manage map assets",
            Tab::Ref => "Read game DB/INI/REF files and output as JSON",
            Tab::Database => "Populate a local SQLite database from game fixtures",
            Tab::Sprite => "Parse .SPR / .SPX files to extract frames or sequences",
            Tab::Sound => "Convert .SNF game audio to standard .WAV format",
        })
        .size(13)
        .style(style::subtle_text);

        let content = column![
            header,
            subtitle,
            vertical_space().height(16),
            inner,
            vertical_space().height(16),
            row![horizontal_space(), run_btn].width(Fill),
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
                let btn = if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                };
                btn.into()
            })
            .collect();

        let op_row = row(op_buttons).spacing(6).wrap();

        let fields: Element<Message> = match self.map_op {
            Some(MapOp::Tiles) | Some(MapOp::Sprites) => column![
                labeled_file_row(
                    "Input (.GTL / .BTL / .MAP):",
                    &self.map_input,
                    Message::MapInputChanged,
                    Message::BrowseMapInput
                ),
                labeled_input(
                    "Output directory:",
                    &self.map_output,
                    Message::MapOutputChanged
                ),
            ]
            .spacing(10)
            .into(),
            Some(MapOp::Atlas) => column![
                labeled_file_row(
                    "Input (.GTL / .BTL):",
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
                    .label("Save embedded sprites")
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
                    "GTL Atlas PNG:",
                    &self.map_gtl_atlas,
                    Message::MapGtlAtlasChanged,
                    Message::BrowseMapGtlAtlas
                ),
                labeled_file_row(
                    "BTL Atlas PNG:",
                    &self.map_btl_atlas,
                    Message::MapBtlAtlasChanged,
                    Message::BrowseMapBtlAtlas
                ),
                labeled_input(
                    "Atlas Columns:",
                    &self.map_atlas_columns,
                    Message::MapAtlasColumnsChanged
                ),
                labeled_input("Output PNG:", &self.map_output, Message::MapOutputChanged),
                labeled_file_row(
                    "Game Path (optional):",
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

        column![op_row, vertical_space().height(12), fields]
            .spacing(4)
            .into()
    }

    fn view_ref_tab(&self) -> Element<'_, Message> {
        let picker = pick_list(RefOp::ALL, self.ref_op, Message::RefOpSelected)
            .placeholder("Select a ref type…")
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
                let btn = if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                };
                btn.into()
            })
            .collect();

        let op_row = row(op_buttons).spacing(6).wrap();

        let description = match self.db_op {
            Some(DbOp::Import) => {
                "Imports everything: maps → refs → rest → dialog-texts → databases"
            }
            Some(DbOp::DialogTexts) => "Imports .dlg and .pgp dialogue files",
            Some(DbOp::Maps) => "Imports .map files + AllMap.ini + Map.ini",
            Some(DbOp::Databases) => {
                "Imports .db files (weapons, stores, monsters, items, spells, quests, messages)"
            }
            Some(DbOp::Refs) => "Imports INI config files (Extra, Event, Monster, Npc, Wave)",
            Some(DbOp::Rest) => {
                "Imports .ref and .pgp reference files (party, draw, NPC, monster, extra refs)"
            }
            None => "Select an operation above.",
        };

        column![
            op_row,
            vertical_space().height(12),
            container(text(description).size(13))
                .padding(12)
                .style(style::info_card),
        ]
        .spacing(8)
        .into()
    }

    fn view_sprite_tab(&self) -> Element<'_, Message> {
        let mode_picker = pick_list(
            SpriteMode::ALL,
            self.sprite_mode,
            Message::SpriteModeSelected,
        )
        .padding(8);

        column![
            labeled_file_row(
                "Input (.SPR / .SPX):",
                &self.sprite_input,
                Message::SpriteInputChanged,
                Message::BrowseSpriteInput
            ),
            row![text("Mode:").size(13).width(140), mode_picker]
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

        let header = row![text("Output Log").size(13), horizontal_space(), clear_btn,]
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
}

// ─── Helpers ────────────────────────────────────────────────────────────────

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
            .size(13),
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
            "Process exited with code {}.\n{}",
            output.status.code().unwrap_or(-1),
            result
        ))
    }
}
