use crate::database::{
    save_draw_items, save_edit_items, save_event_items, save_events, save_extra_refs, save_extras,
    save_heal_items, save_map_inis, save_maps, save_misc_items, save_monster_inis,
    save_monster_refs, save_monsters, save_npc_inis, save_npc_refs, save_party_refs, save_stores,
    save_wave_inis, save_weapons,
};
use crate::references::misc_item_db::read_misc_item_db;
use crate::references::npc_ref::read_npc_ref;
use crate::references::references::read_mutli_magic_db;
use database::{save_dialogs, save_party_pgps};
use rusqlite::Connection;
use std::io::{self};
use std::path::{Path, PathBuf};

pub mod database;
pub mod map;
mod references;
pub mod snf;
pub mod sprite;
pub mod tileset;

use crate::references::{
    all_map_ini, dialog, draw_item, edit_item_db, event_ini, event_item_db, extra_ini, extra_ref,
    heal_item_db, map_ini, misc_item_db, monster_db, monster_ini, monster_ref, npc_ini, npc_ref,
    party_pgp, party_ref, store_db, wave_ini, weapons_db,
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
    Map(MapArgs),
    Ref(RefArgs),
    Database(DatabaseArgs),

    Sprite {
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
        mode: SpriteMode,
    },

    Sound {
        input: String,
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
    // Sprites { map_file: String },
    Tiles {
        input: String,
    },
    Atlas {
        input: String,
        output: String,
    },
    Render {
        #[arg(short, long)]
        map: String,

        #[arg(short, long)]
        btl: String,

        #[arg(short, long)]
        gtl: String,

        #[arg(short, long)]
        output: String,

        #[arg(short, long)]
        save_sprites: bool,
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
    AllMaps { input: String },
    Map { input: String },
    Extra { input: String },
    Event { input: String },
    Monster { input: String },
    Npc { input: String },
    Wave { input: String },
    PartyRef { input: String },
    DrawItem { input: String },
    PartyPgp { input: String },
    PartyDialog { input: String },
    Dialog { input: String },
    Weapons { input: String },
    Monsters { input: String },
    MultiMagic { input: String },
    Store { input: String },
    NpcRef { input: String },
    MonsterRef { input: String },
    MiscItem { input: String },
    HealItems { input: String },
    ExtraRef { input: String },
    EventItems { input: String },
    EditItems { input: String },
    PartyLevel { input: String },
    EventNpcRef { input: String },
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
    Import {},
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
                println!("Rendering map into single canvas...");

                let input_map_file = &Path::new(map);
                let input_btl_file = &Path::new(btl);
                let input_gtl_file = &Path::new(gtl);
                let output_path = &Path::new(output);
                map::extract(
                    input_map_file,
                    input_btl_file,
                    input_gtl_file,
                    output_path,
                    save_sprites,
                )
                .expect("ERROR: could not render map");
            }
            None => {}
        },
        Some(Commands::Ref(ref_args)) => {
            match &ref_args.command {
                Some(RefCommands::AllMaps { input }) => {
                    let data = all_map_ini::read_all_map_ini(&Path::new(input))
                        .expect("ERROR: could not read file");
                    println!(
                        "{}",
                        serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                    );
                }
                Some(RefCommands::Map { input }) => {
                    let data = map_ini::read_map_ini(&Path::new(input))
                        .expect("ERROR: could not read file");
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
                    let data = npc_ini::read_npc_ini(&Path::new(input))
                        .expect("ERROR: could not read file");
                    println!(
                        "{}",
                        serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                    );
                }
                Some(RefCommands::Wave { input }) => {
                    let data = wave_ini::read_wave_ini(&Path::new(input))
                        .expect("ERROR: could not read file");
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
                    let data = dialog::read_dialogs(&Path::new(input))
                        .expect("ERROR: could not read file");
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
                    let data = dialog::read_dialogs(&Path::new(input))
                        .expect("ERROR: could not read file");
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
                    let data = store_db::read_store_db(&Path::new(input))
                        .expect("ERROR: could not read file");
                    println!(
                        "{}",
                        serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                    );
                }
                Some(RefCommands::EventNpcRef { input }) => {
                    // todo let event_npc_refs = references::read_event_npc_ref(&Path::new("sample-data/NpcInGame/Eventnpc.ref"))?;
                }
                Some(RefCommands::NpcRef { input }) => {
                    let data = npc_ref::read_npc_ref(&Path::new(input))
                        .expect("ERROR: could not read file");
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
                    // let party_level = references::read_party_level_db(&Path::new("sample-data/NpcInGame/PrtLevel.db"))?;
                }
                None => {}
            }
        }
        Some(Commands::Database(database_args)) => match &database_args.command {
            Some(DatabaseCommands::Import {}) => {
                save_all().expect("ERROR: could not import all data")
            }
            None => {}
        },
        None => {}
    }
}

fn save_all() -> io::Result<()> {
    let maps = all_map_ini::read_all_map_ini(&Path::new("sample-data/AllMap.ini"))?;
    let map_inis = map_ini::read_map_ini(&Path::new("sample-data/Ref/Map.ini"))?;
    let extras = extra_ini::read_extra_ini(&Path::new("sample-data/Extra.ini"))?;
    let events = event_ini::read_event_ini(&Path::new("sample-data/Event.ini"))?;
    let monster_inis = monster_ini::read_monster_ini(&Path::new("sample-data/Monster.ini"))?;
    let npc_inis = npc_ini::read_npc_ini(&Path::new("sample-data/Npc.ini"))?;
    let wave_inis = wave_ini::read_wave_ini(&Path::new("sample-data/Wave.ini"))?;
    let party_refs = party_ref::read_part_refs(&Path::new("sample-data/Ref/PartyRef.ref"))?;
    let draw_items = draw_item::read_draw_items(&Path::new("sample-data/Ref/DRAWITEM.ref"))?;
    let party_pgps = party_pgp::read_party_pgps(&Path::new("sample-data/NpcInGame/PartyPgp.pgp"))?;
    let dialogs = dialog::read_dialogs(&Path::new("sample-data/NpcInGame/Dlgcat1.dlg"))?;

    let weapons =
        weapons_db::read_weapons_db(&Path::new("sample-data/CharacterInGame/weaponItem.db"))?;
    let stores = store_db::read_store_db(&Path::new("sample-data/CharacterInGame/STORE.DB"))?;
    let npcrefs = npc_ref::read_npc_ref(&Path::new("sample-data/NpcInGame/Npccat1.ref"))?;
    let monsters = monster_db::read_monster_db(&Path::new("sample-data/MonsterInGame/Monster.db"))?;
    let monster_refs =
        monster_ref::read_monster_ref(&Path::new("sample-data/MonsterInGame/Mondun01.ref"))?;
    let misc_items =
        misc_item_db::read_misc_item_db(&Path::new("sample-data/CharacterInGame/MiscItem.db"))?;
    let heal_items =
        heal_item_db::read_heal_item_db(&Path::new("sample-data/CharacterInGame/HealItem.db"))?;
    let extra_refs = extra_ref::read_extra_ref(&Path::new("sample-data/ExtraInGame/Extdun01.ref"))?;
    let event_items =
        event_item_db::read_event_item_db(&Path::new("sample-data/CharacterInGame/EventItem.db"))?;
    let edit_items =
        edit_item_db::read_edit_item_db(&Path::new("sample-data/CharacterInGame/EditItem.db"))?;

    let conn = Connection::open("database.sqlite").unwrap();

    save_maps(&conn, &maps).unwrap();
    save_events(&conn, &events).unwrap();
    save_extras(&conn, &extras).unwrap();
    save_monster_inis(&conn, &monster_inis).unwrap();
    save_npc_inis(&conn, &npc_inis).unwrap();
    save_wave_inis(&conn, &wave_inis).unwrap();
    save_map_inis(&conn, &map_inis).unwrap();
    save_party_refs(&conn, &party_refs).unwrap();
    save_draw_items(&conn, &draw_items).unwrap();
    save_party_pgps(&conn, &party_pgps).unwrap();
    save_dialogs(&conn, &dialogs).unwrap();

    save_monsters(&conn, &monsters).unwrap();
    save_stores(&conn, &stores).unwrap();
    save_weapons(&conn, &weapons).unwrap();
    save_npc_refs(&conn, &npcrefs).unwrap();
    save_monster_refs(&conn, &monster_refs).unwrap();
    save_misc_items(&conn, &misc_items).unwrap();
    save_heal_items(&conn, &heal_items).unwrap();
    save_extra_refs(&conn, &extra_refs).unwrap();
    save_event_items(&conn, &event_items).unwrap();
    save_edit_items(&conn, &edit_items).unwrap();

    conn.close().unwrap();

    Ok(())
}
