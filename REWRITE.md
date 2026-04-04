Code Quality Audit Report: src/
1. DUPLICATED CODE PATTERNS
1.1 Massive .expect() duplication in ref_command.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/ref_command.rs, lines 53-281
- Description: Every single ref subcommand branch follows the identical pattern of read_xxx().expect("ERROR: could not read file") followed by println!("{}", serde_json::to_string(&data).expect("ERROR: could not encode JSON")). This is copy-pasted 28 times with only the read function and module changing.
- Suggested fix: Create a helper function like fn print_json_result<T: Serialize>(data: T) -> Result<(), Box<dyn Error>> and call it from each branch. Even better, use the Extractor trait to dispatch dynamically instead of a massive match.
- Severity: HIGH
1.2 Massive .expect("Command execution failed") duplication in main.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/main.rs, lines 412-724
- Description: Every command dispatch arm ends with command.execute().expect("Command execution failed"). This appears 12 times.
- Suggested fix: Extract to a helper: fn run_command(cmd: impl Command) { cmd.execute().expect("Command execution failed"); } or better, propagate errors properly with fn main() -> Result<(), Box<dyn Error>>.
- Severity: HIGH
1.3 Duplicate enum-to-enum mapping in main.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/main.rs, lines 448-506 and 514-673
- Description: The MapCommands -> commands::map::MapSubcommand mapping and RefCommands -> commands::ref_command::RefSubcommand mapping are essentially identity transformations where every field is .clone()d into a structurally identical enum variant. The RefCommands match for the deprecation hint (lines 514-544) is also a near-duplicate of the actual dispatch match (lines 549-673).
- Suggested fix: Make the CLI enums and command enums the same type, or derive a conversion trait. The deprecated hint match should be generated from the same data.
- Severity: MEDIUM
1.4 Duplicate save_all and individual import functions in commands/database.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/database.rs, lines 87-103
- Description: save_all() calls import_maps(), import_refs(), import_rest(), import_dialog_texts(), import_databases() -- but each of the sub-commands (lines 54-73) also opens a connection and calls the same function. The Connection::open(db_path)? pattern is repeated 6 times.
- Suggested fix: Create a helper fn with_connection(db_path: &str, f: impl FnOnce(&mut Connection) -> Result<()>) to deduplicate connection management.
- Severity: MEDIUM
1.5 Duplicate Connection::open + map_err pattern in map/database.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/database.rs, lines 39-44
- Description: image::open(...).map_err(|e| std::io::Error::other(e.to_string())) appears 2 times, Connection::open(...).map_err(|e| std::io::Error::other(e.to_string())) appears 1 time, and .map_err(|e| std::io::Error::other(e.to_string())) appears 10+ times throughout the file.
- Suggested fix: Create a helper fn io_err(e: impl std::fmt::Display) -> std::io::Error { std::io::Error::other(e.to_string()) } or use .map_err(std::io::Error::other) directly where possible.
- Severity: LOW
1.6 Duplicate get_type_fields hardcoded field lists in info.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/info.rs, lines 520-654
- Description: get_type_fields() contains hardcoded field lists for each file type (weapons, monsters, all_maps, map_ini, etc.). These are duplicated from the actual struct definitions in the reference modules.
- Suggested fix: Derive field names from the actual structs using a trait or macro, or generate this from the Extractor trait.
- Severity: MEDIUM
1.7 Duplicate make_*() function structure in registry.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/registry.rs, lines 176-590
- Description: Every make_*() function follows the exact same pattern: construct a FileType with key, name, description, extensions, detect_kind, extract_fn, patch_fn, validate_fn. There are 26 such functions.
- Suggested fix: Create a builder or macro to reduce boilerplate. A FileTypeBuilder struct or make_file_type! macro would cut this in half.
- Severity: MEDIUM
1.8 Duplicate save_* database function pattern in reference modules
- File: All *_db.rs and *_ini.rs files in references/
- Description: Every save_* function follows the pattern: let tx = conn.transaction()?; { let mut stmt = tx.prepare(include_str!("../queries/..."))?; for record in records { stmt.execute(params![...])?; } } tx.commit()?;. This is repeated in weapons_db.rs, monster_db.rs, all_map_ini.rs, store_db.rs, and many others.
- Suggested fix: Create a generic fn batch_insert<T>(conn: &mut Connection, sql: &str, records: &[T], to_params: impl Fn(&T) -> Vec<Box<dyn ToSql>>) helper.
- Severity: MEDIUM
1.9 Duplicate read_file boilerplate in reference parsers
- File: All *_db.rs files in references/
- Description: Every binary DB parser starts with the identical pattern: File::open, file.metadata()?.len(), BufReader::new(file), read_mapper(...). This is in weapons_db.rs, monster_db.rs, store_db.rs, misc_item_db.rs, etc.
- Suggested fix: Create a helper fn open_db_reader(path: &Path, counter_size: u8, item_size: i32) -> io::Result<(BufReader<File>, i32)>.
- Severity: MEDIUM
---
2. ANTI-PATTERNS AND CODE SMELLS
2.1 Box<dyn Error> everywhere -- missing typed error types
- Files: commands/mod.rs:20, commands/unified.rs, commands/info.rs, commands/map.rs, commands/ref_command.rs, commands/sprite.rs, commands/sound.rs, commands/database.rs, commands/test.rs
- Description: The Command trait uses Result<(), Box<dyn Error>>. This makes it impossible for callers to match on specific error types. All command implementations use string-formatting for errors via .map_err(|e| format!(...)).
- Suggested fix: Define a proper CommandError enum with variants like FileNotFound, ParseError, IoError(io::Error), JsonError(serde_json::Error), etc.
- Severity: HIGH
2.2 .expect() used for all command execution in main.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/main.rs, lines 412, 416, 420, 424, 428, 432, 440, 444, 508, 675, 719, 724
- Description: All 12 command executions use .expect("Command execution failed"), which will panic on any user error (missing file, bad JSON, etc.). A CLI tool should never panic on user errors.
- Suggested fix: Change fn main() to fn main() -> Result<(), Box<dyn Error>> or use eprintln! + std::process::exit(1).
- Severity: HIGH
2.3 .expect() in ref_command.rs for every parser call
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/ref_command.rs, lines 55-279
- Description: 28 .expect("ERROR: could not read file") and 28 .expect("ERROR: could not encode JSON") calls. Any file I/O error or serialization error will panic.
- Suggested fix: Use ? operator and return the error through the Command::execute Result.
- Severity: HIGH
2.4 .expect() in commands/map.rs for all subcommands
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/map.rs, lines 61, 71, 90, 112, 118, 127
- Description: 6 .expect() calls for map operations. Tile extraction, map rendering, database import, and sprite extraction all panic on failure.
- Suggested fix: Propagate errors with ?.
- Severity: HIGH
2.5 .expect() in commands/sprite.rs and commands/sound.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/sprite.rs, lines 23, 26, 39, 42
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/sound.rs, line 16
- Description: All sprite and sound operations use .expect() instead of error propagation.
- Suggested fix: Use ? operator.
- Severity: HIGH
2.6 .unwrap() in map/mod.rs for file_stem() and to_str()
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/mod.rs, lines 428, 485, 488, 536
- Description: input_map_file.file_stem().unwrap().to_str().unwrap() will panic if the file has no stem or the stem is not valid UTF-8.
- Suggested fix: Use .ok_or_else(|| io::Error::new(...)) or return a proper error.
- Severity: MEDIUM
2.7 .unwrap() in map/render.rs for imgbuf.save()
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/render.rs, line 94
- Description: imgbuf.save(output_path).unwrap() will panic if the output path is not writable.
- Suggested fix: Use .map_err(...) to propagate the error.
- Severity: MEDIUM
2.8 .unwrap() in map/tileset.rs for image saves and try_into()
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/tileset.rs, lines 178, 210, 211, 258, 260, 266, 297, 298
- Description: Multiple .unwrap() calls for imgbuf.save() and try_into(). The try_into().unwrap() for coordinate conversion will panic if coordinates overflow u32.
- Suggested fix: Handle errors gracefully, especially for coordinate overflow which could happen with malformed input.
- Severity: MEDIUM
2.9 .unwrap() in sprite.rs for image saves
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/sprite.rs, lines 434, 459
- Description: imgbuf.save(...).unwrap() in save_sequence_anim and save_sequence.
- Suggested fix: Propagate the error through the Result<()> return type.
- Severity: MEDIUM
2.10 .unwrap() in commands/database.rs for conn.close() and file_stem()
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/database.rs, lines 100, 152
- Description: conn.close().unwrap() ignores close errors. path.file_stem().unwrap().to_str().unwrap() panics on invalid filenames.
- Suggested fix: Use let _ = conn.close(); if ignoring is intentional, or handle the error. Use proper error propagation for file stem.
- Severity: LOW
2.11 unimplemented!() in map/reader.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/reader.rs, line 57
- Description: unimplemented!("Unexpected image-stamp {image_stamp}") will panic at runtime if an unexpected image stamp value is encountered in a map file.
- Suggested fix: Return an io::Error::new(InvalidData, ...) instead.
- Severity: MEDIUM
2.12 Unnecessary .clone() on String arguments in main.rs
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/main.rs, ~70 instances
- Description: The entire dispatch match is filled with .clone() calls because the CLI args are borrowed from &cli.command but the command constructors take owned String. This is a consequence of the command factory pattern taking owned values.
- Suggested fix: Change command constructors to accept &str or impl AsRef<str> and clone internally only when needed.
- Severity: LOW
2.13 &Vec<T> instead of &[T] in save functions
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/references/weapons_db.rs:253, monster_db.rs:356, all_map_ini.rs:154, store_db.rs:233, message_scr.rs:147, and many others
- Description: Many save_* functions take &Vec<T> instead of the more idiomatic &[T]. This unnecessarily restricts callers.
- Suggested fix: Change all &Vec<T> parameters to &[T].
- Severity: LOW
2.14 Bug in mix_color -- blue channel uses red
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/tileset.rs, line 327
- Description: let b: u8 = ((color.r as f64 * amount) + base.b as f64 * (1.0 - amount)) as u8; -- uses color.r instead of color.b for the blue channel calculation.
- Suggested fix: Change color.r to color.b.
- Severity: HIGH (actual bug)
2.15 Two different rgb16_565_produce_color implementations
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/sprite.rs, lines 122-136
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/tileset.rs, lines 342-353
- Description: There are two implementations of RGB565 decoding that produce different results. The sprite.rs version uses bit-shifts (red_value << 3) which gives values like 248 for max red. The tileset.rs version uses floating-point scaling (r as f32 * 255.0 / 31.0).round() which gives 255 for max red. The tests in each module assert different expected values.
- Suggested fix: Consolidate to a single implementation. The floating-point version is more accurate for full-range 8-bit output.
- Severity: MEDIUM
2.16 TILE_PIXEL_NUMBER defined twice
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/types.rs, line 9
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/tileset.rs, line 64
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/tileset.rs, line 319 (inside mix_color)
- Description: TILE_PIXEL_NUMBER is defined in both types.rs and tileset.rs with the same value. It's also re-declared as a local constant inside mix_color.
- Suggested fix: Define once in types.rs and import where needed. Remove the local constant in mix_color.
- Severity: LOW
2.17 CommandFactory._services is unused
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/mod.rs, line 33
- Description: CommandFactory has a _services: services::ServiceContainer field that is never used. The ServiceContainer is an empty struct with no functionality.
- Suggested fix: Remove the field and the entire services.rs file until DI is actually needed.
- Severity: LOW
2.18 Command::name() and Command::description() are dead code
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/mod.rs, lines 22-28
- Description: Both name() and description() are marked #[allow(dead_code)] and are never called anywhere in the codebase.
- Suggested fix: Remove them or actually use them (e.g., for help output).
- Severity: LOW
2.19 MultiMagic case inconsistency
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/ref_command.rs, line 150
- Description: The function is called read_mutli_magic_db (typo: "mutli" instead of "multi"). Also, the RefSubcommand::MultiMagic variant doesn't print JSON like all the others -- it just prints "MultiMagic DB processed successfully" and discards the result.
- Suggested fix: Rename to read_multi_magic_db. Make it consistent with other ref commands.
- Severity: LOW
2.20 &bool parameter instead of bool
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/map/mod.rs, line 424
- Description: pub fn extract(..., save_map_sprites: &bool) takes &bool instead of bool. bool is Copy and passing by reference is unidiomatic.
- Suggested fix: Change to save_map_sprites: bool.
- Severity: LOW
2.21 db_path: &String instead of &str
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/database.rs, line 87
- Description: fn save_all(game_path: &Path, db_path: &String) takes &String instead of &str.
- Suggested fix: Change to db_path: &str.
- Severity: LOW
2.22 Missing #[must_use] on pure functions
- Files: sprite.rs (rgb16_565_produce_color, compute_rect, compute_frame_offset), map/types.rs (convert_map_coords_to_image_coords)
- Description: Pure functions that return computed values have no #[must_use] attribute, so their return values can be silently discarded.
- Suggested fix: Add #[must_use] to pure utility functions.
- Severity: LOW
2.23 Public fields on all structs
- Files: Throughout the codebase
- Description: Nearly every struct has all fields pub. This breaks encapsulation and makes it impossible to enforce invariants. Examples: MapData (mod.rs:125), MapModel (model.rs:10), FileType (registry.rs:24), all reference structs.
- Suggested fix: Make fields private where possible and provide accessor methods. At minimum, mark fields that are implementation details as pub(crate).
- Severity: LOW
---
3. STRUCTURAL ISSUES
3.1 main.rs is 728 lines -- too large
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/main.rs
- Description: The main file contains CLI struct definitions, all subcommand enums, and the entire dispatch logic. It should be split into a cli.rs module for argument definitions.
- Suggested fix: Move Cli, Commands, MapArgs, MapCommands, RefArgs, RefCommands, DatabaseArgs, DatabaseCommands, SpriteMode into src/cli.rs.
- Severity: MEDIUM
3.2 commands/database.rs is 444 lines with hardcoded file lists
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/database.rs
- Description: Contains hardcoded arrays of 30+ map files (lines 114-148), 14 dialog files (lines 214-229), 15 PGP files (lines 236-252), 26 monster ref files (lines 366-396), 28 extra ref files (lines 405-436), and 9 NPC ref files (lines 342-352). These should be discovered dynamically or loaded from configuration.
- Suggested fix: Use directory scanning with glob patterns instead of hardcoded file lists.
- Severity: MEDIUM
3.3 commands/info.rs is 682 lines
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/commands/info.rs
- Description: Contains 4 command implementations, schema generation, template generation, field extraction, and output types. The get_type_fields function alone is 135 lines of hardcoded data.
- Suggested fix: Split schema generation and template generation into separate files. Move get_type_fields to a data file or generate it.
- Severity: MEDIUM
3.4 commands/registry.rs is 706 lines
- File: /Users/piotr/Projects/dispel-re/dispel-re/dispel-extractor/src/commands/registry.rs
- Description: Contains the registry, 26 make_* factory functions, generic helpers, and detection helpers. The factory functions are highly repetitive.
- Suggested fix: Split factory functions into a separate file, or use a macro/data-driven approach.
- Severity: LOW
3.5 Inconsistent re-export patterns
- Files: lib.rs vs map/mod.rs
- Description: lib.rs re-exports individual types from references (lines 14-44), but map/mod.rs re-exports its entire public surface (lines 96-101). The references module has no re-exports at all -- consumers must use full paths like references::weapons_db::WeaponItem.
- Suggested fix: Be consistent. Either re-export all public types at the module level, or require full paths everywhere.
- Severity: LOW
3.6 lib.rs and main.rs both declare modules
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/lib.rs, lines 6-11
- File: /Users/piotr/Projects/dispel-re/dispel-extractor/src/main.rs, lines 3-9
- Description: Both lib.rs and main.rs declare the same modules (database, map, references, snf, sprite). This is intentional for a binary+library crate, but main.rs also declares mod commands which is not in lib.rs.
- Suggested fix: This is actually correct for the pattern. No change needed, but worth noting that commands is only available to the binary, not the library.
- Severity: INFO
3.7 Duplicate Commands enum and RefSubcommand enum
- File: main.rs (RefCommands enum, lines 304-363) vs commands/ref_command.rs (RefSubcommand enum, lines 18-48)
- Description: The CLI defines RefCommands and the command layer defines RefSubcommand with the same variants. The mapping between them is a manual 1:1 copy in main.rs lines 549-673.
- Suggested fix: Use a single enum type shared between CLI and command layers, or use a derive macro to generate the conversion.
- Severity: MEDIUM
3.8 Same pattern for DatabaseCommands and DatabaseSubcommand
- File: main.rs (lines 374-390) vs commands/database.rs (lines 39-46)
- Description: Same issue as 3.7 -- duplicate enum with manual mapping.
- Suggested fix: Same as 3.7.
- Severity: MEDIUM
3.9 MapCommands and MapSubcommand duplication
- File: main.rs (lines 163-293) vs commands/map.rs (lines 12-50)
- Description: Same issue -- duplicate enum with manual mapping in main.rs lines 448-506.
- Suggested fix: Same as 3.7.
- Severity: MEDIUM
---
4. SUMMARY BY SEVERITY
HIGH (10 issues)
1. .expect() duplication in ref_command.rs (28+ instances) -- commands/ref_command.rs:53-281
2. .expect() duplication in main.rs (12 instances) -- main.rs:412-724
3. Box<dyn Error> everywhere, no typed errors -- all command files
4. .expect() for all command execution in main.rs -- main.rs:412-724
5. .expect() in ref_command.rs for all parsers -- commands/ref_command.rs:55-279
6. .expect() in commands/map.rs for all subcommands -- commands/map.rs:61-127
7. .expect() in commands/sprite.rs and commands/sound.rs -- commands/sprite.rs:23-42, commands/sound.rs:16
8. Bug: mix_color blue channel uses color.r instead of color.b -- map/tileset.rs:327
9. Two conflicting rgb16_565_produce_color implementations -- sprite.rs:122 vs tileset.rs:342
10. Command::execute returns Box<dyn Error> preventing typed error handling -- commands/mod.rs:20
MEDIUM (14 issues)
1. Duplicate enum-to-enum mapping in main.rs -- main.rs:448-673
2. Duplicate Connection::open pattern in commands/database.rs -- commands/database.rs:54-73
3. Hardcoded field lists in get_type_fields -- commands/info.rs:520-654
4. Repetitive make_*() functions in registry.rs -- commands/registry.rs:176-590
5. Duplicate save_* database function patterns -- all reference modules
6. Duplicate read_file boilerplate in reference parsers -- all *_db.rs files
7. .unwrap() on file_stem()/to_str() in map/mod.rs -- map/mod.rs:428,485,488,536
8. .unwrap() on imgbuf.save() in map/render.rs -- map/render.rs:94
9. .unwrap() on imgbuf.save() and try_into() in map/tileset.rs -- map/tileset.rs:178,210,211,258,260,266,297,298
10. .unwrap() on imgbuf.save() in sprite.rs -- sprite.rs:434,459
11. unimplemented!() in map/reader.rs -- map/reader.rs:57
12. main.rs is 728 lines -- should split CLI definitions
13. commands/database.rs has hardcoded file lists (100+ entries) -- commands/database.rs:114-436
14. Duplicate CLI/command enum pairs (RefCommands/RefSubcommand, DatabaseCommands/DatabaseSubcommand, MapCommands/MapSubcommand)
LOW (12 issues)
1. ~70 unnecessary .clone() calls in main.rs dispatch
2. &Vec<T> instead of &[T] in save functions (multiple files)
3. TILE_PIXEL_NUMBER defined 3 times
4. Unused CommandFactory._services field -- commands/mod.rs:33
5. Dead code Command::name() and Command::description() -- commands/mod.rs:22-28
6. Typo read_mutli_magic_db -- commands/ref_command.rs:150
7. &bool parameter instead of bool -- map/mod.rs:424
8. &String parameter instead of &str -- commands/database.rs:87
9. Missing #[must_use] on pure functions
10. All struct fields are pub -- no encapsulation
11. Inconsistent re-export patterns between modules
12. Duplicate map_err(|e| std::io::Error::other(e.to_string())) pattern -- map/database.rs
