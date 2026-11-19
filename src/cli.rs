//! Declarative CLI surface.
//!
//! Every top-level subcommand is declared exactly once in `define_commands!`
//! below. That single declaration is the source of truth for:
//!   * the clap `Commands` enum used for parsing,
//!   * the `COMMAND_SPECS` table (name / about / long_about) that backs
//!     `--help` and is validated against the Makefile by the unit tests, and
//!   * `dispatch()`, which consumes the *owned* parsed arguments — no
//!     `args.clone()` and no hand-rebuilt payloads in `main.rs`.
//!
//! Argument structs live here too, next to the declaration that uses them.
//! The behaviour (`Command::execute`) lives in `crate::commands::*`.

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

// ---------------------------------------------------------------------------
// Argument types — one per subcommand
// ---------------------------------------------------------------------------

/// `extract`: read a game file and emit annotated JSON.
#[derive(Args, Clone)]
pub struct ExtractArgs {
    /// Path to game file
    #[arg(short, long)]
    pub input: String,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    pub output: Option<String>,

    /// File type override
    #[arg(short, long)]
    pub r#type: Option<String>,

    /// Pretty-print JSON
    #[arg(short, long)]
    pub pretty: bool,
}

/// `patch`: write JSON data back into a game file.
#[derive(Args, Clone)]
pub struct PatchArgs {
    /// Path to JSON file
    #[arg(short, long)]
    pub input: String,

    /// Path to game file to patch
    #[arg(short, long)]
    pub target: String,

    /// Output path (if different from target)
    #[arg(short, long)]
    pub output: Option<String>,

    /// File type override
    #[arg(long)]
    pub r#type: Option<String>,

    /// Validate without writing
    #[arg(short, long)]
    pub dry_run: bool,

    /// Patch target directly, create .bak backup
    #[arg(long)]
    pub in_place: bool,

    /// Skip backup creation (with --in-place)
    #[arg(long)]
    pub no_backup: bool,
}

/// `validate`: validate JSON against the registered file-type schema.
#[derive(Args, Clone)]
pub struct ValidateArgs {
    /// Path to JSON file
    #[arg(short, long)]
    pub input: String,

    /// File type
    #[arg(long)]
    pub r#type: String,

    /// Verbose output
    #[arg(long)]
    pub verbose: bool,
}

/// `list`: show supported file types.
#[derive(Args, Clone)]
pub struct ListArgs {
    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Filter types by name
    #[arg(long)]
    pub filter: Option<String>,
}

/// `schema`: emit a JSON Schema for a file type.
#[derive(Args, Clone)]
pub struct SchemaArgs {
    /// File type
    #[arg(long)]
    pub r#type: String,
}

/// `template`: emit a minimal record template for a file type.
#[derive(Args, Clone)]
pub struct TemplateArgs {
    /// File type
    #[arg(long)]
    pub r#type: String,

    /// Pretty-print JSON
    #[arg(short, long)]
    pub pretty: bool,
}

/// `mod-pack`: batch-export event scripts to JSON.
#[derive(Args, Clone)]
pub struct ModPackArgs {
    /// Path to the Dispel game directory (contains Ref/Event*.scr)
    #[arg(short, long)]
    pub game_path: String,

    /// Output directory for JSON files
    #[arg(short, long, default_value = "mod-pack")]
    pub output: String,

    /// Pretty-print JSON
    #[arg(short, long)]
    pub pretty: bool,

    /// Output as a single JSON array file instead of individual files
    #[arg(short, long)]
    pub single_file: bool,
}

/// `sprite`: extract frames or sequences from an SPR file.
#[derive(Args, Clone)]
pub struct SpriteArgs {
    /// Path to the source .SPR file
    pub input: String,

    /// Mode: 'sprite' (individual frames) or 'animation' (full sequence)
    #[arg(long, value_name = "MODE", default_value_t = SpriteMode::Sprite, value_enum)]
    pub mode: SpriteMode,

    /// Render animation frames at their raw top-left position instead of aligning their origins
    #[arg(long)]
    pub no_origin_offset: bool,

    /// Keep raw 0x0000 pixels as opaque #000000 instead of making them transparent
    #[arg(long)]
    pub preserve_black: bool,

    /// Output sprite metadata as JSON (no rendering)
    #[arg(long)]
    pub info: bool,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpriteMode {
    Sprite,
    Animation,
}

/// `sound`: audio conversion (subcommand selects the direction).
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct SoundArgs {
    #[command(subcommand)]
    pub command: Option<SoundCommands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SoundCommands {
    /// Convert SNF to WAV
    ToWav {
        /// Source .SNF file
        input: String,
        /// Destination .WAV file
        output: String,
    },
    /// Convert WAV to SNF
    FromWav {
        /// Source .WAV file
        input: String,
        /// Destination .SNF file
        output: String,
    },
}

/// `dialog`: print the dialog flow of a DLG/PGP pair.
#[derive(Args, Clone)]
pub struct DialogArgs {
    /// Path to the DLG file (dialog configuration)
    #[arg(value_name = "DLG_FILE")]
    pub dlg_path: PathBuf,

    /// Path to the corresponding PGP file (dialog text/paragraphs)
    #[arg(short, long, value_name = "PGP_FILE")]
    pub pgp_path: Option<PathBuf>,

    /// Path to the NPC ref file (for linking NPC names to dialogs)
    #[arg(short = 'n', long = "npc-ref", value_name = "NPC_REF_FILE")]
    pub npc_ref_path: Option<PathBuf>,

    /// Path to SQLite database for event information
    #[arg(
        short = 'd',
        long = "database",
        alias = "database-path",
        value_name = "DATABASE_FILE"
    )]
    pub database_path: Option<PathBuf>,
}

/// `map`: map extraction / rendering / conversion (subcommand selects the operation).
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
        /// Draw the tiles' black pixels instead of skipping them. The ground
        /// pass is an opaque row-copy; the BTL pass uses
        /// the skip-zero blit for the shadowed refs whose per-ref blend byte
        /// is non-zero. Defaults to the tileset kind: opaque for .GTL,
        /// skip-zero for .BTL.
        #[arg(long)]
        opaque: Option<bool>,
    },
    /// Render a full map from binary data
    #[command(
        about = "Render complete game map",
        long_about = "Synthesizes the ground layer (GTL), building layer (BTL), and sprites into a single high-resolution image. All layers are enabled by default; use --no-* flags to disable specific layers."
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
        /// Path to the Dispel game directory (enables entity overlay for NPCs, monsters, extras)
        #[arg(long)]
        game_path: Option<String>,

        // Layer visibility toggles (all default to true)
        /// Disable ground (GTL) tile layer
        #[arg(long)]
        no_ground: bool,
        /// Disable buildings (tiled objects / BTL objects layer)
        #[arg(long)]
        no_buildings: bool,
        /// Disable roof (BTL) tile layer
        #[arg(long)]
        no_roofs: bool,
        /// Disable the shadow/lighting pass (fog-of-war darkening on maps
        /// flagged Dark in AllMap.ini)
        #[arg(long)]
        no_shadows: bool,
        /// Path to the 500-byte viewport light template (500×1 PNG). The game
        /// rewrites every tile's light level from this table each frame; the
        /// map file's own bits are dead data, so a dark map renders pitch
        /// black without it.
        #[arg(long)]
        light_template: Option<String>,
        /// Camera tile the light template is anchored at ("x,y"). Defaults to
        /// the middle of the map.
        #[arg(long)]
        camera: Option<String>,
        /// Disable internal sprites (embedded in .map file)
        #[arg(long)]
        no_internal_sprites: bool,
        /// Disable external monster rendering
        #[arg(long)]
        no_monsters: bool,
        /// Disable external NPC rendering
        #[arg(long)]
        no_npcs: bool,
        /// Disable external objects (extras) rendering
        #[arg(long)]
        no_objects: bool,
        /// Animation frame index for internal sprites (default 0; clamped to
        /// each sequence's last frame)
        #[arg(long)]
        frame: Option<usize>,

        // Canvas mode
        /// Output RGBA PNG with alpha channel (black background pixels become transparent)
        #[arg(long)]
        transparent: bool,

        // Overlay toggles
        /// Show collision overlay (red blended diamonds on blocked tiles)
        #[arg(long)]
        collisions: bool,
        /// Show event overlay (magenta blended circles with white event ID labels)
        #[arg(long)]
        events: bool,
        /// Show draw items overlay (coloured diamonds by item type with white ID labels: red=weapon, green=heal, blue=edit, magenta=event, yellow=misc, grey=other)
        #[arg(long)]
        draw_items: bool,
        /// Show NPC waypoint arrows (coloured per waypoint slot: green→blue→red→yellow, with index labels)
        #[arg(long)]
        npc_waypoints: bool,
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
    /// Export map to Tiled TMX format
    #[command(
        about = "Export map to Tiled TMX format",
        long_about = "Exports a .MAP file to Tiled's isometric TMX format with tileset PNGs.\n\nUsage Examples:\n  dispel-extractor map tmx --map cat1.map --gtl cat1.gtl --btl cat1.btl --output out/"
    )]
    Tmx {
        /// Path to the .MAP file
        #[arg(short, long)]
        map: String,
        /// Path to the .GTL ground tileset file
        #[arg(short, long)]
        gtl: String,
        /// Path to the .BTL building tileset file
        #[arg(short, long)]
        btl: String,
        /// Output directory for TMX file and tileset PNGs
        #[arg(short, long, default_value = "tmx_out")]
        output: String,
    },
}

/// `database`: SQLite population (subcommand selects the import stage).
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
    /// Import all SPR files
    Sprites { game_path: String, db_path: String },
}

// ---------------------------------------------------------------------------
// The command specification
// ---------------------------------------------------------------------------

/// A single top-level command as advertised by `--list-commands` / `--help`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub about: &'static str,
}

/// Declares the whole CLI surface once and derives everything that consumes it:
/// the clap enum, the static spec table and the owning dispatch function.
macro_rules! define_commands {
    ($(
        $variant:ident {
            name: $name:literal,
            about: $about:literal,
            long_about: $long:literal,
            args: $args:ty,
        }
    ),* $(,)?) => {
        #[derive(Subcommand)]
        pub enum Commands {
            $(
                #[command(name = $name, about = $about, long_about = $long)]
                $variant($args),
            )*
        }

        /// Source of truth for command names and help text (validated in tests).
        pub static COMMAND_SPECS: &[CommandSpec] = &[
            $(CommandSpec { name: $name, about: $about }),*
        ];

        /// Route an owned, parsed command to its `Command::execute` implementation.
        pub fn dispatch(command: Option<Commands>) -> Result<(), Box<dyn Error>> {
            let Some(command) = command else {
                return Err(crate::commands::missing_subcommand("dispel-extractor"));
            };
            match command {
                $(Commands::$variant(args) => crate::commands::Command::execute(&args)),*
            }
        }
    };
}

define_commands! {
    Extract {
        name: "extract",
        about: "Extract game file data to JSON",
        long_about: "Reads game reference files and outputs their contents as JSON.\n\nUsage Examples:\n  dispel-extractor extract -i fixtures/Dispel/Monster.ini\n  dispel-extractor extract -i fixtures/Dispel/CharacterInGame/weaponItem.db -o weapons.json --pretty",
        args: ExtractArgs,
    },
    Patch {
        name: "patch",
        about: "Patch game files from JSON data",
        long_about: "Writes JSON data back to game binary files.\n\nUsage Examples:\n  dispel-extractor patch -i weapons.json -t fixtures/Dispel/CharacterInGame/weaponItem.db --in-place",
        args: PatchArgs,
    },
    Validate {
        name: "validate",
        about: "Validate JSON against file format",
        long_about: "Validates JSON data against a file format schema.\n\nUsage Examples:\n  dispel-extractor validate -i weapons.json --type weapons",
        args: ValidateArgs,
    },
    List {
        name: "list",
        about: "List supported file types",
        long_about: "Lists all supported file types with descriptions.\n\nUsage Examples:\n  dispel-extractor list\n  dispel-extractor list --format json --filter monster",
        args: ListArgs,
    },
    Schema {
        name: "schema",
        about: "Generate JSON Schema for a file type",
        long_about: "Outputs a JSON Schema describing the structure of a file type's records.\n\nUsage Examples:\n  dispel-extractor schema --type weapons",
        args: SchemaArgs,
    },
    Template {
        name: "template",
        about: "Generate a minimal JSON template for a file type",
        long_about: "Outputs a minimal JSON template for a single record of a file type.\n\nUsage Examples:\n  dispel-extractor template --type weapons --pretty",
        args: TemplateArgs,
    },
    Map {
        name: "map",
        about: "Extract and render map assets",
        long_about: "Operations for handling binary .MAP files and their associated .GTL/.BTL tilesets.\n\nUsage Examples:\n  dispel-extractor map tiles cat1.gtl\n  dispel-extractor map atlas cat1.gtl atlas.png\n  dispel-extractor map render --map cat1.map --btl cat1.btl --gtl cat1.gtl --output map.png",
        args: MapArgs,
    },
    Database {
        name: "database",
        about: "Populate SQLite database",
        long_about: "Initializes and populates a local 'database.sqlite' using the hardcoded paths for game fixtures.\n\nUsage Examples:\n  dispel-extractor database import",
        args: DatabaseArgs,
    },
    Sprite {
        name: "sprite",
        about: "Extract frames or sequences from SPR files",
        long_about: "Parses .SPR (Sprite)\n\nUsage Examples:\n  dispel-extractor sprite character.spr\n  dispel-extractor sprite animation_effect.spr --mode animation\n  dispel-extractor sprite character.spr --preserve-black\n  dispel-extractor sprite character.spr --info",
        args: SpriteArgs,
    },
    Sound {
        name: "sound",
        about: "Convert SNF/WAV audio files",
        long_about: "Convert SNF to WAV or WAV to SNF.\n\nExamples:\n  dispel-extractor sound to-wav track.snf track.wav\n  dispel-extractor sound from-wav track.wav track.snf",
        args: SoundArgs,
    },
    Dialog {
        name: "dialog",
        about: "Print dialog flow from DLG and PGP files",
        long_about: "Reads a DLG (dialog configuration) file and optionally a PGP (dialog text) file, then prints the complete dialog flow as a tree.\n\nUsage Examples:\n  dispel-extractor dialog fixtures/Dispel/NpcInGame/Dlgcat1.dlg\n  dispel-extractor dialog fixtures/Dispel/NpcInGame/Dlgcat1.dlg -p fixtures/Dispel/NpcInGame/Pgpcat1.pgp\n  dispel-extractor dialog fixtures/Dispel/NpcInGame/Dlgcat1.dlg -p fixtures/Dispel/NpcInGame/Pgpcat1.pgp -n fixtures/Dispel/NpcInGame/Npccat1.ref\n  dispel-extractor dialog fixtures/Dispel/NpcInGame/Dlgcat1.dlg -p fixtures/Dispel/NpcInGame/Pgpcat1.pgp -n fixtures/Dispel/NpcInGame/Npccat1.ref --database database.sqlite",
        args: DialogArgs,
    },
    ModPack {
        name: "mod-pack",
        about: "Batch export event scripts (.scr) to JSON",
        long_about: "Reads all Event*.scr files from a Dispel game directory's Ref/ folder and exports them as JSON for use in Godot (or other tools).\n\nUsage Examples:\n  dispel-extractor mod-pack --game-path fixtures/Dispel --output mod-pack/\n  dispel-extractor mod-pack -g fixtures/Dispel -o mod-pack/ --pretty\n  dispel-extractor mod-pack -g fixtures/Dispel -o mod-pack/ --single-file",
        args: ModPackArgs,
    },
}

#[derive(Parser)]
#[command(about = "Tool to extract and patch Dispel game files")]
#[command(author, version, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Print the registered command surface (machine-friendly) and exit
    #[arg(long)]
    pub list_commands: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Machine-friendly listing of the command surface, generated from the spec.
/// The Makefile's `command-list` target uses this instead of a hand-maintained
/// list of subcommands.
pub fn print_command_spec() {
    for spec in COMMAND_SPECS {
        println!("{:<12} {}", spec.name, spec.about);
    }
}

// ---------------------------------------------------------------------------
// Tests: spec <-> clap <-> Makefile, plus in-process `Command::execute` runs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;
    use crate::commands::missing_subcommand;
    use clap::CommandFactory;
    use std::collections::BTreeSet;

    /// Parses a command fully in-process: no subprocess, no network, no fixtures.
    fn run<C: Command>(cmd: &C) -> Result<(), Box<dyn Error>> {
        cmd.execute()
    }

    const MAKEFILE: &str = include_str!("../Makefile");

    /// Snapshot of the advertised command surface. Adding, removing or renaming
    /// a command must be an explicit decision — update this list with the spec.
    #[test]
    fn test_command_spec_snapshot() {
        let names: Vec<&str> = COMMAND_SPECS.iter().map(|spec| spec.name).collect();
        assert_eq!(
            names,
            [
                "extract", "patch", "validate", "list", "schema", "template", "map", "database",
                "sprite", "sound", "dialog", "mod-pack"
            ]
        );
    }

    /// `--help` must expose exactly the commands declared in the spec.
    #[test]
    fn test_command_spec_matches_clap_subcommands() {
        let clap_names: BTreeSet<String> = Cli::command()
            .get_subcommands()
            .map(|cmd| cmd.get_name().to_owned())
            .collect();
        let spec_names: BTreeSet<String> = COMMAND_SPECS
            .iter()
            .map(|spec| spec.name.to_owned())
            .collect();
        assert_eq!(clap_names, spec_names);
    }

    /// Each spec entry must carry non-empty help text and a unique name.
    #[test]
    fn test_command_spec_help_text_is_complete_and_unique() {
        let mut seen = BTreeSet::new();
        for spec in COMMAND_SPECS {
            assert!(!spec.name.is_empty());
            assert!(!spec.about.is_empty(), "missing about for {}", spec.name);
            assert!(
                seen.insert(spec.name),
                "duplicate spec entry: {}",
                spec.name
            );
        }
    }

    /// `--list-commands` parses without a subcommand and mirrors the spec.
    #[test]
    fn test_list_commands_flag() {
        let parsed = Cli::try_parse_from(["dispel-extractor", "--list-commands"])
            .expect("--list-commands must parse without a subcommand");
        assert!(parsed.list_commands);
        assert!(parsed.command.is_none());

        let listing: String = COMMAND_SPECS
            .iter()
            .map(|spec| format!("{:<12} {}\n", spec.name, spec.about))
            .collect();
        assert_eq!(listing.lines().count(), COMMAND_SPECS.len());
        assert!(listing.starts_with("extract      Extract game file data to JSON"));
    }

    /// The Makefile is a second consumer of the command surface: every
    /// `cargo run -- <cmd>` invocation must name a registered command.
    #[test]
    fn test_makefile_invocations_reference_registered_commands() {
        let invoked: Vec<&str> = MAKEFILE
            .lines()
            .filter_map(|line| line.split_once("cargo run -- ").map(|(_, rest)| rest))
            .filter_map(|rest| rest.split_whitespace().next())
            .filter(|token| !token.starts_with('-')) // e.g. `cargo run -- --help`
            .collect();

        assert!(!invoked.is_empty(), "no CLI invocations found in Makefile");
        for cmd in invoked {
            assert!(
                COMMAND_SPECS.iter().any(|spec| spec.name == cmd),
                "Makefile invokes unregistered CLI command '{cmd}'"
            );
        }
    }

    /// Regression guards for concrete Makefile drift found in the wild.
    #[test]
    fn test_makefile_drift_regressions() {
        // Sound conversion must state the direction explicitly.
        assert!(
            MAKEFILE.contains("sound to-wav"),
            "make sound must use `sound to-wav`"
        );
        assert!(
            !MAKEFILE.contains("sound --input"),
            "sound has no --input/--output flags"
        );
        // `dialog` advertises `--database` in its own long_about.
        assert!(
            MAKEFILE.contains("--database \"$(db_path)\""),
            "make dialogs must use the documented --database flag"
        );
        assert!(
            !MAKEFILE.contains("--database-path"),
            "--database-path is the legacy alias, not the documented flag"
        );
        // There is no `test` subcommand; the smoke target must use a real one.
        assert!(!MAKEFILE.contains("cargo run -- test "));
    }

    /// All three parents with nested subcommands share one error message.
    #[test]
    fn test_missing_subcommand_errors_are_consistent() {
        let expected = missing_subcommand("map").to_string();
        assert_eq!(
            run(&MapArgs { command: None }).unwrap_err().to_string(),
            expected
        );
        assert_eq!(
            run(&DatabaseArgs { command: None })
                .unwrap_err()
                .to_string(),
            missing_subcommand("database").to_string()
        );
        assert_eq!(
            run(&SoundArgs { command: None }).unwrap_err().to_string(),
            missing_subcommand("sound").to_string()
        );
        assert!(
            expected.contains("requires a subcommand"),
            "unexpected error text: {expected}"
        );
    }

    /// A few real `Command::execute` invocations through the in-process harness.
    #[test]
    fn test_execute_commands_in_process() {
        // Pure, read-only commands: no game files or network needed.
        run(&ListArgs {
            format: "json".to_owned(),
            filter: Some("monster".to_owned()),
        })
        .expect("list must succeed");
        run(&SchemaArgs {
            r#type: "weapons".to_owned(),
        })
        .expect("schema must succeed");
        run(&TemplateArgs {
            r#type: "weapons".to_owned(),
            pretty: false,
        })
        .expect("template must succeed");

        // Missing-input paths are reported as errors, never silently ignored.
        let err = run(&ExtractArgs {
            input: "/definitely/not/a/real/file.db".to_owned(),
            output: None,
            r#type: None,
            pretty: false,
        })
        .expect_err("extract on a missing file must fail");
        assert!(err.to_string().contains("File not found"), "{err}");

        let err = run(&ValidateArgs {
            input: "/definitely/not/a/real/file.json".to_owned(),
            r#type: "weapons".to_owned(),
            verbose: false,
        })
        .expect_err("validate on a missing file must fail");
        assert!(err.to_string().contains("File not found"), "{err}");
    }

    /// `dispatch` takes ownership and routes every variant without cloning.
    #[test]
    fn test_dispatch_owns_arguments() {
        dispatch(Some(Commands::List(ListArgs {
            format: "json".to_owned(),
            filter: None,
        })))
        .expect("dispatch must succeed");
        dispatch(Some(Commands::Map(MapArgs { command: None })))
            .expect_err("dispatch must surface the missing-subcommand error");
        dispatch(None).expect_err("dispatch must reject a missing root subcommand");
    }
}
