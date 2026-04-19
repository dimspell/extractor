# Dispel Game File Extractor

A **research tool** for modding Dispel game files. Written in Rust, this project focuses on analyzing file formats, data structures, and technical specifications for educational and interoperability purposes.

⚠️ **Important Legal Notice**: This tool is for **educational and research purposes only**. It does not distribute copyrighted game content and complies with fair use principles for reverse engineering research.

---

## 🎯 Purpose

The tool focuses exclusively on **technical specifications** and does not include or distribute any copyrighted game assets.


## 🛠️ Features

| Feature | Description |
|--------|-------------|
| **Map Rendering** | Render full game maps from `.map`, `.btl`, and `.gtl` files |
| **Tile Extraction** | Extract individual tiles from tileset files |
| **Sprite Extraction** | Extract character sprites and animations from `.SPR` files |
| **Sound Conversion** | Convert `.snf` audio files to standard `.wav` format |
| **Database Import** | Import game data into SQLite database for analysis |
| **Reference Parsing** | Parse `.ini`, `.db`, `.ref` files and export as JSON |
| **Atlas Generation** | Generate sprite atlases for visualization |
| **Unified Workflow** | Extract, patch, and validate game files using JSON |
| **Schema Generation** | Generate JSON schemas for file types |
| **Template Generation** | Create minimal JSON templates for editing |
| **Dialogue Support** | Parse and extract `.dlg` and `.pgp` dialogue files |
| **GUI Editor** | Full-featured graphical editor with 27 editor types |

## 🚀 Installation & Usage

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)
- Git

### Build Instructions

```bash
# Build the project
cargo build --release

# Or build with debug symbols for development
cargo build
```

### Running Commands

```bash
# View help
./target/release/dispel-extractor --help

# View command-specific help
./target/release/dispel-extractor extract --help
./target/release/dispel-extractor patch --help
./target/release/dispel-extractor validate --help
```

### Running the GUI

```bash
# Build and run the GUI application
cargo run -p dispel-gui
```

---

## 🗺️ Map Extraction

Extract and render game maps from `.map`, `.btl`, and `.gtl` files.

### Map Rendering

Render a complete map to a PNG image:

```bash
# Render a specific map
./target/release/dispel-extractor map render \
    --map fixtures/Dispel/Map/cat1.map \
    --btl fixtures/Dispel/Map/cat1.btl \
    --gtl fixtures/Dispel/Map/cat1.gtl \
    --output map_render.png
```

### Extract Tiles from Tileset

```bash
# Extract individual tiles from a tileset
./target/release/dispel-extractor map tiles \
    fixtures/Dispel/Map/cat1.gtl \
    --output out/tiles/
```

### Generate Sprite Atlas

```bash
# Create an atlas image from a tileset
./target/release/dispel-extractor map atlas \
    fixtures/Dispel/Map/cat1.btl \
    cat1_atlas.png
```

### Extract Map Sprites

```bash
# Extract sprites used in a map
./target/release/dispel-extractor map sprites \
    fixtures/Dispel/Map/cat1.map \
    --output out/cat1_sprites/
```

---

## 🎨 Sprite & Animation Extraction

Extract character sprites and animations from `.SPR` files.

```bash
# Extract a sprite sheet
./target/release/dispel-extractor sprite \
    fixtures/Dispel/CharacterInGame/M_BODY1.SPR \
    --mode sprite

# Extract animations
./target/release/dispel-extractor sprite \
    fixtures/Dispel/CharacterInGame/M_BODY1.SPR \
    --mode animation
```

---

## 🔊 Sound Conversion

Convert Dispel `.snf` audio files to standard `.wav` format.

```bash
# Convert a sound file
./target/release/dispel-extractor sound \
    --input fixtures/Dispel/Sound/sample.snf \
    --output output.wav
```

---

## 🗃️ Database & References

### Extract Game Data to JSON

```bash
# Extract a specific file to JSON
./target/release/dispel-extractor extract -i fixtures/Dispel/Monster.ini

# Extract with pretty formatting
./target/release/dispel-extractor extract -i fixtures/Dispel/CharacterInGame/weaponItem.db --pretty
```

### Patch Game Files from JSON

```bash
# Patch a file in-place
./target/release/dispel-extractor patch -i weapons.json -t fixtures/Dispel/CharacterInGame/weaponItem.db --in-place
```

### Validate JSON Against Schema

```bash
# Validate JSON data
./target/release/dispel-extractor validate -i weapons.json --type weapons
```

### List Supported File Types

```bash
# List all supported file types
./target/release/dispel-extractor list

# Filter by type
./target/release/dispel-extractor list --filter monster
```

### Generate JSON Schema

```bash
# Generate schema for a file type
./target/release/dispel-extractor schema --type weapons
```

### Generate JSON Template

```bash
# Generate a minimal template
./target/release/dispel-extractor template --type weapons --pretty
```

---

## 📜 License

This research tool is licensed under the **[MIT License](LICENSE)** for the **code only**.

### License Scope

✅ **Applies to**: All source code in this repository
❌ **Does not apply to**: Game content, assets, or proprietary formats

---

🔒 **This project operates under fair use principles for reverse engineering research and educational purposes.**
