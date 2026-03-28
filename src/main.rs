use std::path::PathBuf;

pub mod database;
pub mod map;
pub mod references;
pub mod snf;
pub mod sprite;

mod commands;

use clap::{Args, Parser, Subcommand, ValueEnum};
use commands::{Command, CommandFactory};

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

    /// Test command
    #[command(
        about = "Test command for verifying the command pattern",
        long_about = "A simple test command to verify the command pattern implementation.\n\nUsage Examples:\n  dispel-extractor test --message 'Hello World'"
    )]
    Test {
        /// Test message to display
        #[arg(short, long, default_value = "Hello from test command!")]
        message: String,
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

        /// Output directory to save the tile images
        #[arg(short, long, default_value = "out")]
        output: String,
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

        /// Path to the Dispel game directory (enables sprite rendering for NPCs, monsters, extras)
        #[arg(long)]
        game_path: Option<String>,
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
    /// Extract map internal sprites
    #[command(
        about = "Extract map internal sprites",
        long_about = "Extracts all internal sprites embedded in a .MAP file to separate PNG files."
    )]
    Sprites {
        /// Path to the .MAP file
        input: String,

        /// Output directory to save the sprites
        #[arg(short, long, default_value = "out")]
        output: String,
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

    // Create command factory with dependency injection
    let command_factory = CommandFactory::new();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Sprite { input, mode }) => {
            let mode_enum = match mode {
                SpriteMode::Sprite => commands::sprite::SpriteMode::Sprite,
                SpriteMode::Animation => commands::sprite::SpriteMode::Animation,
            };
            let command = command_factory.create_sprite_command(input.clone(), mode_enum);
            command.execute().expect("Command execution failed");
        }
        Some(Commands::Sound { input, output }) => {
            let command = command_factory.create_sound_command(input.clone(), output.clone());
            command.execute().expect("Command execution failed");
        }
        Some(Commands::Map(map_args)) => {
            if let Some(map_command) = &map_args.command {
                let subcommand = match map_command {
                    MapCommands::Tiles { input, output } => commands::map::MapSubcommand::Tiles {
                        input: input.clone(),
                        output: output.clone(),
                    },
                    MapCommands::Atlas { input, output } => commands::map::MapSubcommand::Atlas {
                        input: input.clone(),
                        output: output.clone(),
                    },
                    MapCommands::Render {
                        map,
                        btl,
                        gtl,
                        output,
                        save_sprites,
                    } => commands::map::MapSubcommand::Render {
                        map: map.clone(),
                        btl: btl.clone(),
                        gtl: gtl.clone(),
                        output: output.clone(),
                        save_sprites: *save_sprites,
                    },
                    MapCommands::FromDb {
                        database,
                        map_id,
                        gtl_atlas,
                        btl_atlas,
                        atlas_columns,
                        output,
                        game_path,
                    } => commands::map::MapSubcommand::FromDb {
                        database: database.clone(),
                        map_id: map_id.clone(),
                        gtl_atlas: gtl_atlas.clone(),
                        btl_atlas: btl_atlas.clone(),
                        atlas_columns: *atlas_columns,
                        output: output.clone(),
                        game_path: game_path.clone(),
                    },
                    MapCommands::ToDb { database, map } => commands::map::MapSubcommand::ToDb {
                        database: database.clone(),
                        map: map.clone(),
                    },
                    MapCommands::Sprites { input, output } => {
                        commands::map::MapSubcommand::Sprites {
                            input: input.clone(),
                            output: output.clone(),
                        }
                    }
                };
                let command = command_factory.create_map_command(subcommand);
                command.execute().expect("Command execution failed");
            }
        }
        Some(Commands::Ref(ref_args)) => {
            if let Some(ref_command) = &ref_args.command {
                let subcommand = match ref_command {
                    RefCommands::AllMaps { input } => {
                        commands::ref_command::RefSubcommand::AllMaps {
                            input: input.clone(),
                        }
                    }
                    RefCommands::Map { input } => commands::ref_command::RefSubcommand::Map {
                        input: input.clone(),
                    },
                    RefCommands::Extra { input } => commands::ref_command::RefSubcommand::Extra {
                        input: input.clone(),
                    },
                    RefCommands::Event { input } => commands::ref_command::RefSubcommand::Event {
                        input: input.clone(),
                    },
                    RefCommands::Monster { input } => {
                        commands::ref_command::RefSubcommand::Monster {
                            input: input.clone(),
                        }
                    }
                    RefCommands::Npc { input } => commands::ref_command::RefSubcommand::Npc {
                        input: input.clone(),
                    },
                    RefCommands::Wave { input } => commands::ref_command::RefSubcommand::Wave {
                        input: input.clone(),
                    },
                    RefCommands::DrawItem { input } => {
                        commands::ref_command::RefSubcommand::DrawItem {
                            input: input.clone(),
                        }
                    }
                    RefCommands::Dialog { input } => commands::ref_command::RefSubcommand::Dialog {
                        input: input.clone(),
                    },
                    RefCommands::PartyRef { input } => {
                        commands::ref_command::RefSubcommand::PartyRef {
                            input: input.clone(),
                        }
                    }
                    RefCommands::PartyPgp { input } => {
                        commands::ref_command::RefSubcommand::PartyPgp {
                            input: input.clone(),
                        }
                    }
                    RefCommands::PartyDialog { input } => {
                        commands::ref_command::RefSubcommand::PartyDialog {
                            input: input.clone(),
                        }
                    }
                    RefCommands::Weapons { input } => {
                        commands::ref_command::RefSubcommand::Weapons {
                            input: input.clone(),
                        }
                    }
                    RefCommands::MultiMagic { input } => {
                        commands::ref_command::RefSubcommand::MultiMagic {
                            input: input.clone(),
                        }
                    }
                    RefCommands::Store { input } => commands::ref_command::RefSubcommand::Store {
                        input: input.clone(),
                    },
                    RefCommands::EventNpcRef { input } => {
                        commands::ref_command::RefSubcommand::EventNpcRef {
                            input: input.clone(),
                        }
                    }
                    RefCommands::NpcRef { input } => commands::ref_command::RefSubcommand::NpcRef {
                        input: input.clone(),
                    },
                    RefCommands::Monsters { input } => {
                        commands::ref_command::RefSubcommand::Monsters {
                            input: input.clone(),
                        }
                    }
                    RefCommands::MonsterRef { input } => {
                        commands::ref_command::RefSubcommand::MonsterRef {
                            input: input.clone(),
                        }
                    }
                    RefCommands::MiscItem { input } => {
                        commands::ref_command::RefSubcommand::MiscItem {
                            input: input.clone(),
                        }
                    }
                    RefCommands::HealItems { input } => {
                        commands::ref_command::RefSubcommand::HealItems {
                            input: input.clone(),
                        }
                    }
                    RefCommands::ExtraRef { input } => {
                        commands::ref_command::RefSubcommand::ExtraRef {
                            input: input.clone(),
                        }
                    }
                    RefCommands::EventItems { input } => {
                        commands::ref_command::RefSubcommand::EventItems {
                            input: input.clone(),
                        }
                    }
                    RefCommands::EditItems { input } => {
                        commands::ref_command::RefSubcommand::EditItems {
                            input: input.clone(),
                        }
                    }
                    RefCommands::PartyLevel { input } => {
                        commands::ref_command::RefSubcommand::PartyLevel {
                            input: input.clone(),
                        }
                    }
                    RefCommands::PartyIni { input } => {
                        commands::ref_command::RefSubcommand::PartyIni {
                            input: input.clone(),
                        }
                    }
                    RefCommands::Magic { input } => commands::ref_command::RefSubcommand::Magic {
                        input: input.clone(),
                    },
                    RefCommands::Quest { input } => commands::ref_command::RefSubcommand::Quest {
                        input: input.clone(),
                    },
                    RefCommands::Message { input } => {
                        commands::ref_command::RefSubcommand::Message {
                            input: input.clone(),
                        }
                    }
                    RefCommands::ChData { input } => commands::ref_command::RefSubcommand::ChData {
                        input: input.clone(),
                    },
                };
                let command = command_factory.create_ref_command(subcommand);
                command.execute().expect("Command execution failed");
            }
        }
        Some(Commands::Database(database_args)) => {
            if let Some(database_command) = &database_args.command {
                let subcommand = match database_command {
                    DatabaseCommands::Import {} => commands::database::DatabaseSubcommand::Import,
                    DatabaseCommands::DialogTexts {} => {
                        commands::database::DatabaseSubcommand::DialogTexts
                    }
                    DatabaseCommands::Maps {} => commands::database::DatabaseSubcommand::Maps,
                    DatabaseCommands::Databases {} => {
                        commands::database::DatabaseSubcommand::Databases
                    }
                    DatabaseCommands::Refs {} => commands::database::DatabaseSubcommand::Refs,
                    DatabaseCommands::Rest {} => commands::database::DatabaseSubcommand::Rest,
                };
                let command = command_factory.create_database_command(subcommand);
                command.execute().expect("Command execution failed");
            }
        }
        Some(Commands::Test { message }) => {
            let command = command_factory.create_test_command(message.clone());
            command.execute().expect("Command execution failed");
        }
        None => {}
    }
}
