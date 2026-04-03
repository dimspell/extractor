use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::commands::list::ListArgs;
use crate::commands::schema::SchemaArgs;
use crate::commands::template::TemplateArgs;
use crate::commands::unified::{ExtractArgs, PatchArgs};
use crate::commands::validate::ValidateArgs;

#[derive(Parser)]
#[command(about = "Tool to extract assets from the Dispel game")]
#[command(author, version, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    pub name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract game file data to JSON
    #[command(
        about = "Extract game file data to JSON",
        long_about = "Reads game reference files and outputs their contents as JSON.\n\nUsage Examples:\n  dispel-extractor extract -i fixtures/Dispel/Monster.ini\n  dispel-extractor extract -i fixtures/Dispel/CharacterInGame/weaponItem.db -o weapons.json --pretty"
    )]
    Extract(ExtractArgs),

    /// Patch game files from JSON data
    #[command(
        about = "Patch game files from JSON data",
        long_about = "Writes JSON data back to game binary files.\n\nUsage Examples:\n  dispel-extractor patch -i weapons.json -t fixtures/Dispel/CharacterInGame/weaponItem.db --in-place"
    )]
    Patch(PatchArgs),

    /// Validate JSON against file format
    #[command(
        about = "Validate JSON against file format",
        long_about = "Validates JSON data against a file format schema.\n\nUsage Examples:\n  dispel-extractor validate -i weapons.json --type weapons"
    )]
    Validate(ValidateArgs),

    /// List supported file types
    #[command(
        about = "List supported file types",
        long_about = "Lists all supported file types with descriptions.\n\nUsage Examples:\n  dispel-extractor list\n  dispel-extractor list --format json --filter monster"
    )]
    List(ListArgs),

    /// Generate JSON Schema for a file type
    #[command(
        about = "Generate JSON Schema for a file type",
        long_about = "Outputs a JSON Schema describing the structure of a file type's records.\n\nUsage Examples:\n  dispel-extractor schema --type weapons"
    )]
    Schema(SchemaArgs),

    /// Generate a minimal JSON template for a file type
    #[command(
        about = "Generate a minimal JSON template for a file type",
        long_about = "Outputs a minimal JSON template for a single record of a file type.\n\nUsage Examples:\n  dispel-extractor template --type weapons --pretty"
    )]
    Template(TemplateArgs),

    /// Map operations (tiles, atlas, render)
    #[command(
        about = "Extract and render map assets",
        long_about = "Operations for handling binary .MAP files and their associated .GTL/.BTL tilesets.\n\nUsage Examples:\n  dispel-extractor map tiles cat1.gtl\n  dispel-extractor map atlas cat1.gtl atlas.png\n  dispel-extractor map render --map cat1.map --btl cat1.btl --gtl cat1.gtl --output map.png"
    )]
    Map(MapArgs),

    /// Reference data extraction (deprecated, use extract)
    #[command(
        about = "Convert game DB/INI/REF files to JSON (deprecated, use extract)",
        long_about = "DEPRECATED: Use 'extract' instead.\n\nReads internal game reference files and outputs their contents as JSON for external analysis.\n\nUsage Examples:\n  dispel-extractor ref monster fixtures/Dispel/Monster.ini\n  dispel-extractor ref weapons fixtures/Dispel/CharacterInGame/weaponItem.db"
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
        long_about = "Parses .SPR (Sprite) \n\nUsage Examples:\n  dispel-extractor sprite character.spr\n  dispel-extractor sprite animation_effect.spr --mode animation\n  dispel-extractor sprite character.spr --info"
    )]
    Sprite {
        /// Path to the source .SPR file
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
        /// Output sprite metadata as JSON (no rendering)
        #[arg(long)]
        info: bool,
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
pub enum SpriteMode {
    Sprite,
    Animation,
}

// --------------------------------------------------------------------------
// Map subcommands
// --------------------------------------------------------------------------

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct MapArgs {
    #[command(subcommand)]
    pub command: Option<MapCommands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum MapCommands {
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
    /// Extract map data to JSON
    #[command(
        about = "Extract map data to JSON",
        long_about = "Parses a .MAP file and outputs its complete data structure as JSON.\n\nUsage Examples:\n  dispel-extractor map to-json --input cat1.map --output cat1.json\n  dispel-extractor map to-json --input cat1.map --pretty"
    )]
    ToJson {
        /// Path to the .MAP file
        #[arg(short, long)]
        input: String,
        /// Output JSON file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Pretty-print JSON
        #[arg(short, long)]
        pretty: bool,
    },
}

// --------------------------------------------------------------------------
// Ref subcommands (deprecated)
// --------------------------------------------------------------------------

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct RefArgs {
    #[command(subcommand)]
    pub command: Option<RefCommands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum RefCommands {
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
    /// Read Generic .dlg file
    Dialog { input: String },
    /// Read Dlgcat1.pgp
    DialogTexts { input: String },
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

impl RefCommands {
    /// Returns the corresponding extract type key for migration hints.
    pub fn extract_type_key(&self) -> &'static str {
        match self {
            RefCommands::AllMaps { .. } => "all_maps",
            RefCommands::Map { .. } => "map_ini",
            RefCommands::Extra { .. } => "extra_ini",
            RefCommands::Event { .. } => "event_ini",
            RefCommands::Monster { .. } => "monster_ini",
            RefCommands::Npc { .. } => "npc_ini",
            RefCommands::Wave { .. } => "wave_ini",
            RefCommands::DrawItem { .. } => "draw_item",
            RefCommands::Dialog { .. } => "dialog",
            RefCommands::PartyRef { .. } => "party_ref",
            RefCommands::DialogTexts { .. } => "dialog_text",
            RefCommands::Weapons { .. } => "weapons",
            RefCommands::MultiMagic { .. } => "magic",
            RefCommands::Store { .. } => "store",
            RefCommands::EventNpcRef { .. } => "event_npc_ref",
            RefCommands::NpcRef { .. } => "npc_ref",
            RefCommands::Monsters { .. } => "monsters",
            RefCommands::MonsterRef { .. } => "monster_ref",
            RefCommands::MiscItem { .. } => "misc_item",
            RefCommands::HealItems { .. } => "heal_item",
            RefCommands::ExtraRef { .. } => "extra_ref",
            RefCommands::EventItems { .. } => "event_item",
            RefCommands::EditItems { .. } => "edit_item",
            RefCommands::PartyLevel { .. } => "party_level",
            RefCommands::PartyIni { .. } => "party_ini",
            RefCommands::Magic { .. } => "magic",
            RefCommands::Quest { .. } => "quest",
            RefCommands::Message { .. } => "message",
            RefCommands::ChData { .. } => "chdata",
        }
    }
}

// --------------------------------------------------------------------------
// Database subcommands
// --------------------------------------------------------------------------

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct DatabaseArgs {
    #[command(subcommand)]
    pub command: Option<DatabaseCommands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DatabaseCommands {
    #[command(
        about = "Import all reference files to SQLite",
        long_about = "Reads all standard game files from 'fixtures/Dispel' and saves them to 'database.sqlite'."
    )]
    Import { game_path: String, db_path: String },
    /// Import dialog texts only
    DialogTexts { game_path: String, db_path: String },
    /// Import maps and their tiles only
    Maps { game_path: String, db_path: String },
    /// Import item and character databases (.db files)
    Databases { game_path: String, db_path: String },
    /// Import reference configuration files (INI files)
    Refs { game_path: String, db_path: String },
    /// Import the rest (REF/PGP files)
    Rest { game_path: String, db_path: String },
}
