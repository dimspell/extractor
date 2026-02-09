use crate::database::{
    initialize_database, save_dialogs, save_dialogue_texts, save_draw_items, save_edit_items,
    save_event_items, save_event_npc_refs, save_events, save_extra_refs, save_extras,
    save_heal_items, save_magic_spells, save_map_inis, save_maps, save_messages, save_misc_items,
    save_monster_inis, save_monster_refs, save_monsters, save_npc_inis, save_npc_refs,
    save_party_inis, save_party_levels, save_party_pgps, save_party_refs, save_quests, save_stores,
    save_wave_inis, save_weapons,
};
use crate::references::misc_item_db::read_misc_item_db;
use crate::references::party_ini_db::read_party_ini_db;
use crate::references::party_level_db::read_party_level_db;
use crate::references::references::read_mutli_magic_db;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub mod database;
pub mod map;
mod references;
pub mod snf;
pub mod sprite;
pub mod tileset;
use crate::references::{
    all_map_ini, chdata_db, dialog, dialogue_text, draw_item, edit_item_db, event_ini,
    event_item_db, event_npc_ref, extra_ini, extra_ref, heal_item_db, magic_db, map_ini,
    message_scr, misc_item_db, monster_db, monster_ini, monster_ref, npc_ini, npc_ref,
    party_ini_db, party_level_db, party_pgp, party_ref, quest_scr, store_db, wave_ini, weapons_db,
};
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(about = "Tool to extract assets from the Dispel game")]
#[command(author, version, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Map related operations
    #[command(
        about = "Extract and render map assets",
        long_about = "Operations for handling binary .MAP files and their associated .GTL/.BTL tilesets.\n\nUsage Examples:\n  dispel-extractor map tiles cat1.gtl\n  dispel-extractor map atlas cat1.gtl atlas.png\n  dispel-extractor map render --map cat1.map --btl cat1.btl --gtl cat1.gtl --output map.png"
    )]
    Map(MapArgs),

    /// Reference data extraction
    #[command(
        about = "Convert game DB/INI/REF files to JSON",
        long_about = "Reads internal game reference files and outputs their contents as JSON for external analysis.\n\nUsage Examples:\n  dispel-extractor ref monster fixtures/Dispel/Monster.ini\n  dispel-extractor ref weapons fixtures/Dispel/CharacterInGame/weaponItem.db"
    )]
    Ref(RefArgs),

    /// Database operations
    #[command(
        about = "Populate SQLite database",
        long_about = "Initializes and populates a local 'database.sqlite' using the hardcoded paths for game fixtures.\n\nUsage Examples:\n  dispel-extractor database import"
    )]
    Database(DatabaseArgs),

    /// Sprite/Animation extraction
    #[command(
        about = "Extract frames or sequences from SPR files",
        long_about = "Parses .SPR (Sprite) or .SPX (Animated Sprite) files.\n\nUsage Examples:\n  dispel-extractor sprite character.spr\n  dispel-extractor sprite animation_effect.spx --mode animation"
    )]
    Sprite {
        /// Path to the source .SPR or .SPX file
        input: String,
        #[arg(
            long,
            require_equals = true,
            value_name = "MODE",
            num_args = 0..=1,
            default_value_t = SpriteMode::Sprite,
            default_missing_value = "always",
            value_enum
        )]
        /// Mode: 'sprite' (individual frames) or 'animation' (full sequence)
        mode: SpriteMode,
    },

    /// Audio conversion
    #[command(
        about = "Convert SNF audio to WAV",
        long_about = "Extracts the raw PCM data from an .SNF file and wraps it in a standard RIFF WAVE header.\n\nUsage Examples:\n  dispel-extractor sound track01.snf track01.wav"
    )]
    Sound {
        /// Source .SNF file
        input: String,
        /// Destination .WAV file
        output: String,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum SpriteMode {
    Sprite,
    Animation,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct MapArgs {
    #[command(subcommand)]
    command: Option<MapCommands>,
}

#[derive(Debug, Subcommand)]
enum MapCommands {
    /// Extract every tile as a separate image
    #[command(
        about = "Extract tiles to individual files",
        long_about = "Parses a GTL or BTL file and outputs each 32x32 tile as 'image_N.png'."
    )]
    Tiles {
        /// Path to a .GTL or .BTL tileset file
        input: String,
    },
    /// Create a single image containing all tiles in a grid
    #[command(
        about = "Generate a tileset atlas",
        long_about = "Packs all tiles from a tileset into a single large atlas image."
    )]
    Atlas {
        /// Path to a .GTL or .BTL file
        input: String,
        /// File path for the resulting atlas PNG
        output: String,
    },
    /// Render a full map from binary data
    #[command(
        about = "Render complete game map",
        long_about = "Synthesizes the ground layer (GTL), building layer (BTL), and sprites into a single high-resolution image."
    )]
    Render {
        /// The .MAP geography/collision file
        #[arg(short, long)]
        map: String,

        /// The Building Tile Layer set (.BTL)
        #[arg(short, long)]
        btl: String,

        /// The Ground Tile Layer set (.GTL)
        #[arg(short, long)]
        gtl: String,

        /// Path to save the final PNG render
        #[arg(short, long)]
        output: String,

        /// Also export sub-sprites found within the map file
        #[arg(short, long)]
        save_sprites: bool,
    },
    /// Render a map from SQLite database
    #[command(
        about = "Render map from database",
        long_about = "Renders a map image using tile data from SQLite database and atlas PNG files."
    )]
    FromDb {
        /// Path to the SQLite database file
        #[arg(short, long, default_value = "database.sqlite")]
        database: String,

        /// Map ID to render (e.g., "cat1")
        #[arg(short, long)]
        map_id: String,

        /// Path to the ground tileset atlas PNG
        #[arg(long)]
        gtl_atlas: String,

        /// Path to the building/roof tileset atlas PNG
        #[arg(long)]
        btl_atlas: String,

        /// Number of tiles per row in the atlas (default: 48)
        #[arg(long, default_value = "48")]
        atlas_columns: u32,

        /// Path to save the output PNG
        #[arg(short, long)]
        output: String,
    },
    /// Import a map file into SQLite database
    #[command(
        about = "Import map to database",
        long_about = "Parses a .MAP file and saves its geometry, objects, and sprites to the SQLite database."
    )]
    ToDb {
        /// Path to the SQLite database file
        #[arg(short, long, default_value = "database.sqlite")]
        database: String,

        /// Path to the .MAP file
        #[arg(short, long)]
        map: String,
    },
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct RefArgs {
    #[command(subcommand)]
    command: Option<crate::RefCommands>,
}

#[derive(Debug, Subcommand)]
enum RefCommands {
    /// Read AllMap.ini (General Map List)
    AllMaps { input: String },
    /// Read Map.ini (Specific Map Properties)
    Map { input: String },
    /// Read Extra.ini (Interactive Object Types)
    Extra { input: String },
    /// Read Event.ini (Script/Event Mappings)
    Event { input: String },
    /// Read Monster.ini (Monster Visual Refs)
    Monster { input: String },
    /// Read Npc.ini (NPC Visual Refs)
    Npc { input: String },
    /// Read Wave.ini (Audio/SNF References)
    Wave { input: String },
    /// Read PartyRef.ref (Character Definitions)
    PartyRef { input: String },
    /// Read DRAWITEM.ref (Map Placements)
    DrawItem { input: String },
    /// Read PartyPgp.pgp (Party Dialogue)
    PartyPgp { input: String },
    /// Read Dlgcat1.dlg (Dialogue Category 1)
    PartyDialog { input: String },
    /// Read Generic .dlg file
    Dialog { input: String },
    /// Read weaponItem.db (Armor & Weapons)
    Weapons { input: String },
    /// Read Monster.db (Monster Stats)
    Monsters { input: String },
    /// Read MultiMagic.db (Spells)
    MultiMagic { input: String },
    /// Read STORE.DB (Shop inventores)
    Store { input: String },
    /// Read Npccat1.ref (NPC Placements)
    NpcRef { input: String },
    /// Read Mondun01.ref (Monster Placements)
    MonsterRef { input: String },
    /// Read MiscItem.db (Generic Items)
    MiscItem { input: String },
    /// Read HealItem.db (Consumables)
    HealItems { input: String },
    /// Read Extdun01.ref (Special Object Placements)
    ExtraRef { input: String },
    /// Read EventItem.db (Quest Items)
    EventItems { input: String },
    /// Read EditItem.db (Modifiable Items)
    EditItems { input: String },
    /// Read PartyLevel.db (EXP Tables)
    PartyLevel { input: String },
    /// Read PrtIni.db (Party NPC Metadata)
    PartyIni { input: String },
    /// Read EventNpc.ref (Event-specific NPC placements)
    EventNpcRef { input: String },
    /// Read Magic.db (Magic Spells Database)
    Magic { input: String },
    /// Read Quest.scr
    Quest { input: String },
    /// Read Message.scr
    Message { input: String },
    /// Read ChData.db
    ChData { input: String },
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct DatabaseArgs {
    #[command(subcommand)]
    command: Option<crate::DatabaseCommands>,
}

#[derive(Debug, Subcommand)]
enum DatabaseCommands {
    #[command(
        about = "Import all reference files to SQLite",
        long_about = "Reads all standard game files from 'fixtures/Dispel' and saves them to 'database.sqlite'."
    )]
    Import {},
    /// Import dialog texts only
    DialogTexts {},
    /// Import maps and their tiles only
    Maps {},
    /// Import item and character databases (.db files)
    Databases {},
    /// Import reference configuration files (INI files)
    Refs {},
    /// Import the rest (REF/PGP files)
    Rest {},
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Sprite { input, mode }) => {
            println!("Extracting sprite...");
            match mode {
                SpriteMode::Sprite => {
                    sprite::extract(&Path::new(input), "todo_prefix".to_string())
                        .expect("ERROR: could not export sprite");
                }
                SpriteMode::Animation => {
                    sprite::animation(&Path::new(input)).expect("ERROR: could not export sprite");
                }
            }
        }
        Some(Commands::Sound { input, output }) => {
            println!("Extracting sound file to {output}...");
            snf::extract(&Path::new(input), &Path::new(output))
                .expect("ERROR: could not convert SNF file to WAV");
        }
        Some(Commands::Map(map_args)) => match &map_args.command {
            Some(MapCommands::Tiles { input }) => {
                println!("Extracting all tiles to separate tiles...");
                println!("Input file: {input:?}");

                let tiles =
                    tileset::extract(&Path::new(input)).expect("ERROR: could not extract tile-set");
                tileset::plot_all_tiles(&tiles);
            }
            Some(MapCommands::Atlas { input, output }) => {
                println!("Rendering map atlas...");
                println!("Input file: {input:?}");
                println!("Output file: {output:?}");

                let tiles =
                    tileset::extract(&Path::new(input)).expect("ERROR: could not extract tile-set");
                tileset::plot_tileset_map(&tiles, output);
            }
            Some(MapCommands::Render {
                map,
                btl,
                gtl,
                output,
                save_sprites,
            }) => {
                println!("Rendering map...");
                map::extract(
                    &Path::new(map),
                    &Path::new(btl),
                    &Path::new(gtl),
                    &Path::new(output),
                    save_sprites,
                )
                .expect("ERROR: could not render map");
            }
            Some(MapCommands::FromDb {
                database,
                map_id,
                gtl_atlas,
                btl_atlas,
                atlas_columns,
                output,
            }) => {
                println!("Rendering map from database...");
                map::render_from_database(
                    &Path::new(database),
                    map_id,
                    &Path::new(gtl_atlas),
                    &Path::new(btl_atlas),
                    *atlas_columns,
                    &Path::new(output),
                )
                .expect("ERROR: could not render map from database");
            }
            Some(MapCommands::ToDb { database, map }) => {
                println!("Importing map to database...");
                map::import_to_database(&Path::new(database), &Path::new(map))
                    .expect("ERROR: could not import map to database");
            }
            None => {}
        },
        Some(Commands::Ref(ref_args)) => match &ref_args.command {
            Some(RefCommands::AllMaps { input }) => {
                let data = all_map_ini::read_all_map_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Map { input }) => {
                let data =
                    map_ini::read_map_ini(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Extra { input }) => {
                let data = extra_ini::read_extra_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Event { input }) => {
                let data = event_ini::read_event_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Monster { input }) => {
                let data = monster_ini::read_monster_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Npc { input }) => {
                let data =
                    npc_ini::read_npc_ini(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Wave { input }) => {
                let data =
                    wave_ini::read_wave_ini(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::DrawItem { input }) => {
                let data = draw_item::read_draw_items(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Dialog { input }) => {
                let data =
                    dialog::read_dialogs(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::PartyRef { input }) => {
                let data = party_ref::read_part_refs(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::PartyPgp { input }) => {
                let data = party_pgp::read_party_pgps(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::PartyDialog { input }) => {
                let data =
                    dialog::read_dialogs(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Weapons { input }) => {
                let data = weapons_db::read_weapons_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::MultiMagic { input }) => {
                let data =
                    read_mutli_magic_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Store { input }) => {
                let data =
                    store_db::read_store_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::EventNpcRef { input }) => {
                let data = event_npc_ref::read_event_npc_ref(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::NpcRef { input }) => {
                let data =
                    npc_ref::read_npc_ref(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Monsters { input }) => {
                let data = monster_db::read_monster_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::MonsterRef { input }) => {
                let data = monster_ref::read_monster_ref(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::MiscItem { input }) => {
                let data =
                    read_misc_item_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::HealItems { input }) => {
                let data = heal_item_db::read_heal_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::ExtraRef { input }) => {
                let data = extra_ref::read_extra_ref(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::EventItems { input }) => {
                let data = event_item_db::read_event_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::EditItems { input }) => {
                let data = edit_item_db::read_edit_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::PartyLevel { input }) => {
                let data =
                    read_party_level_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::PartyIni { input }) => {
                let data =
                    read_party_ini_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Magic { input }) => {
                let data =
                    magic_db::read_magic_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Quest { input }) => {
                let data =
                    quest_scr::read_quests(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::Message { input }) => {
                let data = message_scr::read_messages(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            Some(RefCommands::ChData { input }) => {
                let data =
                    chdata_db::read_chdata(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            None => {}
        },
        Some(Commands::Database(database_args)) => match &database_args.command {
            Some(DatabaseCommands::Import {}) => {
                save_all().expect("ERROR: could not import all data")
            }
            Some(DatabaseCommands::DialogTexts {}) => {
                let mut conn =
                    Connection::open("database.sqlite").expect("ERROR: could not open database");
                import_dialog_texts(&mut conn).expect("ERROR: could not import dialog texts")
            }
            Some(DatabaseCommands::Maps {}) => {
                let mut conn =
                    Connection::open("database.sqlite").expect("ERROR: could not open database");
                import_maps(&mut conn).expect("ERROR: could not import maps")
            }
            Some(DatabaseCommands::Databases {}) => {
                let mut conn =
                    Connection::open("database.sqlite").expect("ERROR: could not open database");
                import_databases(&mut conn).expect("ERROR: could not import databases")
            }
            Some(DatabaseCommands::Refs {}) => {
                let mut conn =
                    Connection::open("database.sqlite").expect("ERROR: could not open database");
                import_refs(&mut conn).expect("ERROR: could not import refs")
            }
            Some(DatabaseCommands::Rest {}) => {
                let mut conn =
                    Connection::open("database.sqlite").expect("ERROR: could not open database");
                import_rest(&mut conn).expect("ERROR: could not import rest")
            }
            None => {}
        },
        None => {}
    }
}

fn save_all() -> Result<(), Box<dyn std::error::Error>> {
    println!("Saving all data...");

    let mut conn = Connection::open("database.sqlite")?;

    initialize_database(&conn)?;

    import_maps(&mut conn)?;
    import_refs(&mut conn)?;
    import_rest(&mut conn)?;
    import_dialog_texts(&mut conn)?;
    import_databases(&mut conn)?;

    conn.close().unwrap();

    Ok(())
}

fn import_maps(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let main_path = Path::new("fixtures/Dispel");
    println!("Saving maps...");
    let maps = all_map_ini::read_all_map_ini(&main_path.join("AllMap.ini"))?;
    save_maps(conn, &maps)?;

    println!("Importing all .map files...");
    let map_dir = main_path.join("Map");
    if map_dir.exists() {
        let map_files = [
            "cat1.map",
            "cat2.map",
            "cat3.map",
            "catp.map",
            "dun01.map",
            "dun02.map",
            "dun03.map",
            "dun04.map",
            "dun05.map",
            "dun06.map",
            "dun07.map",
            "dun08.map",
            "dun09.map",
            "dun10.map",
            "dun11.map",
            "dun12.map",
            "dun13.map",
            "dun14.map",
            "dun15.map",
            "dun16.map",
            "dun17.map",
            "dun18.map",
            "dun19.map",
            "dun20.map",
            "dun21.map",
            "dun22.map",
            "dun23.map",
            "dun24.map",
            "dun25.map",
            "final.map",
            "map1.map",
            "map2.map",
            "map3.map",
        ];
        for entry in map_files {
            let path = map_dir.join(entry);
            if path.extension().and_then(|s| s.to_str()) == Some("map") {
                let map_id = path.file_stem().unwrap().to_str().unwrap();
                if map_id == "map4" {
                    continue;
                }
                println!("Importing map file: {}", path.display());
                match std::fs::File::open(&path) {
                    Ok(file) => {
                        let mut reader = std::io::BufReader::new(file);
                        match map::read_map_data(&mut reader) {
                            Ok(map_data) => {
                                if let Err(e) = map::save_to_db(conn, map_id, &map_data) {
                                    eprintln!(
                                        "WARNING: could not save map {} to database: {}",
                                        map_id, e
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "WARNING: could not read map data from {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WARNING: could not open map file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    println!("Saving map_inis...");
    let map_inis = map_ini::read_map_ini(&main_path.join("Ref/Map.ini"))?;
    save_map_inis(conn, &map_inis)?;
    Ok(())
}

fn import_refs(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let main_path = Path::new("fixtures/Dispel");
    println!("Saving extras...");
    let extras = extra_ini::read_extra_ini(&main_path.join("Extra.ini"))?;
    save_extras(conn, &extras)?;
    println!("Saving events...");
    let events = event_ini::read_event_ini(&main_path.join("Event.ini"))?;
    save_events(conn, &events)?;
    println!("Saving monster_inis...");
    let monster_inis = monster_ini::read_monster_ini(&main_path.join("Monster.ini"))?;
    save_monster_inis(conn, &monster_inis)?;
    println!("Saving npc_inis...");
    let npc_inis = npc_ini::read_npc_ini(&main_path.join("Npc.ini"))?;
    save_npc_inis(conn, &npc_inis)?;
    println!("Saving wave_inis...");
    let wave_inis = wave_ini::read_wave_ini(&main_path.join("Wave.ini"))?;
    save_wave_inis(conn, &wave_inis)?;
    Ok(())
}

fn import_dialog_texts(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let main_path = Path::new("fixtures/Dispel");
    let dialog_files = [
        "NpcInGame/Dlgcat1.dlg",
        "NpcInGame/Dlgcat2.dlg",
        "NpcInGame/Dlgcat3.dlg",
        "NpcInGame/Dlgcatp.dlg",
        "NpcInGame/Dlgdun04.dlg",
        "NpcInGame/Dlgdun07.dlg",
        "NpcInGame/Dlgdun08.dlg",
        "NpcInGame/Dlgdun10.dlg",
        "NpcInGame/Dlgdun19.dlg",
        "NpcInGame/Dlgdun22.dlg",
        "NpcInGame/Dlgmap1.dlg",
        "NpcInGame/Dlgmap2.dlg",
        "NpcInGame/Dlgmap3.dlg",
        "NpcInGame/PartyDlg.dlg",
    ];
    println!("Saving dialogs...");
    for dialog_file in dialog_files {
        let dialogs = dialog::read_dialogs(&main_path.join(dialog_file))?;
        save_dialogs(conn, dialog_file, &dialogs)?;
    }

    let pgp_files = [
        "NpcInGame/PartyPgp.pgp",
        "NpcInGame/Pgpcat1.pgp",
        "NpcInGame/Pgpcat2.pgp",
        "NpcInGame/Pgpcat3.pgp",
        "NpcInGame/Pgpcatp.pgp",
        "NpcInGame/Pgpdun04.pgp",
        "NpcInGame/Pgpdun07.pgp",
        "NpcInGame/Pgpdun08.pgp",
        "NpcInGame/Pgpdun10.pgp",
        "NpcInGame/Pgpdun19.pgp",
        "NpcInGame/Pgpdun22.pgp",
        "NpcInGame/Pgpmap1.pgp",
        "NpcInGame/Pgpmap2.pgp",
        "NpcInGame/Pgpmap3.pgp",
    ];
    println!("Saving dialogue texts...");
    for pgp_file in pgp_files {
        let texts = dialogue_text::read_dialogue_texts(&main_path.join(pgp_file))?;
        save_dialogue_texts(conn, pgp_file, &texts)?;
    }
    Ok(())
}

fn import_databases(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let main_path = Path::new("fixtures/Dispel");
    println!("Saving weapons...");
    let weapons = weapons_db::read_weapons_db(&main_path.join("CharacterInGame/weaponItem.db"))?;
    save_weapons(conn, &weapons)?;
    println!("Saving stores...");
    let stores = store_db::read_store_db(&main_path.join("CharacterInGame/STORE.DB"))?;
    save_stores(conn, &stores)?;
    println!("Saving monsters...");
    let monsters = monster_db::read_monster_db(&main_path.join("MonsterInGame/Monster.db"))?;
    save_monsters(conn, &monsters)?;
    println!("Saving misc_items...");
    let misc_items =
        misc_item_db::read_misc_item_db(&main_path.join("CharacterInGame/MiscItem.db"))?;
    save_misc_items(conn, &misc_items)?;
    println!("Saving heal_items...");
    let heal_items =
        heal_item_db::read_heal_item_db(&main_path.join("CharacterInGame/HealItem.db"))?;
    save_heal_items(conn, &heal_items)?;
    println!("Saving event_items...");
    let event_items =
        event_item_db::read_event_item_db(&main_path.join("CharacterInGame/EventItem.db"))?;
    save_event_items(conn, &event_items)?;
    println!("Saving edit_items...");
    let edit_items =
        edit_item_db::read_edit_item_db(&main_path.join("CharacterInGame/EditItem.db"))?;
    save_edit_items(conn, &edit_items)?;
    println!("Saving party_level_db...");
    let party_levels =
        party_level_db::read_party_level_db(&main_path.join("NpcInGame/PrtLevel.db"))?;
    save_party_levels(conn, &party_levels)?;
    println!("Saving party_ini_db...");
    let party_inis = party_ini_db::read_party_ini_db(&main_path.join("NpcInGame/PrtIni.db"))?;
    save_party_inis(conn, &party_inis)?;
    println!("Saving magic_spells...");
    let magic_spells = magic_db::read_magic_db(&main_path.join("MagicInGame/Magic.db"))?;
    save_magic_spells(conn, &magic_spells)?;

    println!("Saving quests...");
    let quests = quest_scr::read_quests(&main_path.join("ExtraInGame/Quest.scr"))?;
    save_quests(conn, &quests)?;

    println!("Saving messages...");
    let messages = message_scr::read_messages(&main_path.join("ExtraInGame/Message.scr"))?;
    save_messages(conn, &messages)?;

    Ok(())
}

fn import_rest(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let main_path = Path::new("fixtures/Dispel");
    println!("Saving party_refs...");
    let party_refs = party_ref::read_part_refs(&main_path.join("Ref/PartyRef.ref"))?;
    save_party_refs(conn, &party_refs)?;
    println!("Saving draw_items...");
    let draw_items = draw_item::read_draw_items(&main_path.join("Ref/DRAWITEM.ref"))?;
    save_draw_items(conn, &draw_items)?;
    println!("Saving party_pgps...");
    let party_pgps = party_pgp::read_party_pgps(&main_path.join("NpcInGame/PartyPgp.pgp"))?;
    save_party_pgps(conn, &party_pgps)?;

    let npc_ref_files = [
        "NpcInGame/Npccat1.ref",
        "NpcInGame/Npccat2.ref",
        "NpcInGame/Npccat3.ref",
        "NpcInGame/Npccatp.ref",
        "NpcInGame/npcdun08.ref",
        "NpcInGame/npcdun19.ref",
        "NpcInGame/Npcmap1.ref",
        "NpcInGame/Npcmap2.ref",
        "NpcInGame/Npcmap3.ref",
    ];
    println!("Saving npcrefs...");
    for npc_ref_file in npc_ref_files {
        let npcrefs = npc_ref::read_npc_ref(&main_path.join(npc_ref_file))?;
        save_npc_refs(conn, npc_ref_file, &npcrefs)?;
    }

    println!("Saving event_npc_refs...");
    let event_npc_refs =
        event_npc_ref::read_event_npc_ref(&main_path.join("NpcInGame/Eventnpc.ref"))?;
    save_event_npc_refs(conn, &event_npc_refs)?;

    let monster_ref_files = [
        "MonsterInGame/Mondun01.ref",
        "MonsterInGame/Mondun02.ref",
        "MonsterInGame/mondun03.ref",
        "MonsterInGame/mondun04.ref",
        "MonsterInGame/Mondun05.ref",
        "MonsterInGame/mondun06.ref",
        "MonsterInGame/mondun07.ref",
        "MonsterInGame/mondun08.ref",
        "MonsterInGame/mondun09.ref",
        "MonsterInGame/Mondun10.ref",
        "MonsterInGame/mondun11.ref",
        "MonsterInGame/mondun12.ref",
        "MonsterInGame/mondun13.ref",
        "MonsterInGame/Mondun14.ref",
        "MonsterInGame/mondun15.ref",
        "MonsterInGame/mondun16.ref",
        "MonsterInGame/mondun17.ref",
        "MonsterInGame/mondun18.ref",
        "MonsterInGame/Mondun19.ref",
        "MonsterInGame/mondun20.ref",
        "MonsterInGame/mondun21.ref",
        "MonsterInGame/mondun22.ref",
        "MonsterInGame/mondun23.ref",
        "MonsterInGame/mondun24.ref",
        "MonsterInGame/mondun25.ref",
        "MonsterInGame/monfinal.ref",
        "MonsterInGame/Monmap1.ref",
        "MonsterInGame/Monmap2.ref",
        "MonsterInGame/Monmap3.ref",
    ];
    println!("Saving monster_refs...");
    for monster_ref_file in monster_ref_files {
        let monster_refs = monster_ref::read_monster_ref(&main_path.join(monster_ref_file))?;
        save_monster_refs(conn, monster_ref_file, &monster_refs)?;
    }

    let extra_ref_files = [
        "ExtraInGame/Extcat3.ref",
        "ExtraInGame/Extdun01.ref",
        "ExtraInGame/Extdun02.ref",
        "ExtraInGame/Extdun03.ref",
        "ExtraInGame/Extdun04.ref",
        "ExtraInGame/Extdun05.ref",
        "ExtraInGame/Extdun06.ref",
        "ExtraInGame/Extdun07.ref",
        "ExtraInGame/Extdun08.ref",
        "ExtraInGame/Extdun09.ref",
        "ExtraInGame/Extdun10.ref",
        "ExtraInGame/Extdun11.ref",
        "ExtraInGame/Extdun12.ref",
        "ExtraInGame/Extdun13.ref",
        "ExtraInGame/Extdun14.ref",
        "ExtraInGame/Extdun15.ref",
        "ExtraInGame/Extdun16.ref",
        "ExtraInGame/Extdun17.ref",
        "ExtraInGame/Extdun18.ref",
        "ExtraInGame/Extdun19.ref",
        "ExtraInGame/Extdun20.ref",
        "ExtraInGame/Extdun21.ref",
        "ExtraInGame/Extdun22.ref",
        "ExtraInGame/Extdun23.ref",
        "ExtraInGame/Extdun24.ref",
        "ExtraInGame/Extdun25.ref",
        "ExtraInGame/Extfinal.ref",
        "ExtraInGame/Extmap1.ref",
        "ExtraInGame/Extmap2.ref",
        "ExtraInGame/Extmap3.ref",
    ];
    println!("Saving extra_refs...");
    for extra_ref_file in extra_ref_files {
        let extra_refs = extra_ref::read_extra_ref(&main_path.join(extra_ref_file))?;
        save_extra_refs(conn, extra_ref_file, &extra_refs)?;
    }
    Ok(())
}
