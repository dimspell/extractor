# Dispel Extractor Overview

This project is a tool designed to extract assets and data from "Dispel" game files. It is written in Rust and uses a CLI interface to perform various extraction tasks.

## Project Structure

- **src/main.rs**: The entry point. Handles CLI arguments using `clap` and dispatches commands.
- **src/database.rs**: Handles interaction with a SQLite database to store extracted game data (items, monsters, maps, etc.).
- **src/map.rs**: Logic for parsing binary map files and rendering them to images.
- **src/sprite.rs**: Handles extraction of sprite and animation data.
- **src/tileset.rs**: Logic for extracting and plotting map tiles.
- **src/snf.rs**: Converts SNF audio files to WAV format.
- **src/references/**: (Not fully analyzed but imported) Contains modules for reading various `.ini` and game database files (`.db`, `.ref`).

## CLI Commands

The tool is invoked via command line. The main subcommands are:

### `sprite`
Extracts sprites or animations from a file.
- `input`: Path to the input file.
- `mode`: `sprite` (default) or `animation`.

### `sound`
Converts an SNF sound file to a standard WAV file.
- `input`: Path to source .snf file.
- `output`: Path to destination .wav file.

### `map`
Operations related to map files.
- `tiles`: Extracts separate tiles from a tileset file.
- `atlas`: Renders a tileset into a single atlas image.
- `render`: Renders a full map into a single large image. Use `--save-sprites` to export internal map sprites.

### `ref`
Extracts specific game data reference files (INIs, DBs, REFs) and prints them as JSON.
Supported types:
- `AllMaps`, `Map`, `Extra`, `Event`, `Monster`, `Npc`, `Wave`
- `PartyRef`, `DrawItem`, `PartyPgp`, `Dialog`, `Weapons`, `Store`
- `NpcRef`, `MonsterRef`, `MiscItem`, `HealItems`, `EventItems`, `EditItems`, `PartyLevel`

### `database`
- `import`: Reads a specific set of hardcoded fixtures (game files) and imports them all into a SQLite database (`database.sqlite`).

## Workflow

1.  **Parsing**: The tool reads binary formats (using `byteorder`).
2.  **Processing**: It converts raw data (like 16-bit 565 colors) into usable formats (RGB/RGBA images, SQL records).
3.  **Output**:
    - **Images**: Saved as PNGs.
    - **Audio**: Saved as WAVs.
    - **Data**: Printed as JSON or saved to a SQLite database.
