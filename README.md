# Dispel Game File Extractor

A **research tool** for reverse engineering and extracting data from Dispel game files. Written in Rust, this project focuses on analyzing file formats, data structures, and technical specifications for educational and interoperability purposes.

⚠️ **Important Legal Notice**: This tool is for **educational and research purposes only**. It does not distribute copyrighted game content and complies with fair use principles for reverse engineering research.

---

## 📋 Table of Contents

- [🎯 Purpose](#-purpose)
- [⚖️ Legal Compliance](#%EF%B8%8F-legal-compliance)
- [🛠️ Features](#%EF%B8%8F-features)
- [📦 Project Structure](#-project-structure)
- [🚀 Installation & Usage](#-installation--usage)
- [🗺️ Map Extraction](#%EF%B8%80-map-extraction)
- [🎨 Sprite & Animation Extraction](#-sprite--animation-extraction)
- [🔊 Sound Conversion](#-sound-conversion)
- [🗃️ Database & References](#%EF%B8%80-database--references)
- [📊 Supported File Types](#-supported-file-types)
- [📚 Documentation](#-documentation)
- [🤝 Contributing](#-contributing)
- [📜 License](#-license)

---

## 🎯 Purpose

This project is a **technical research tool** designed to:

- Reverse engineer and document the file formats used by the DISPEL® game
- Extract and analyze game data structures (maps, sprites, items, monsters, etc.)
- Provide educational insights into game file organization and encoding
- Support interoperability research without violating intellectual property rights

The tool focuses exclusively on **technical specifications** and does not include or distribute any copyrighted game assets.

---

## ⚖️ Legal Compliance

### Disclaimer

⚠️ **DISPEL® is a registered trademark** of its respective owner. This project is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Compliance Statement

- ✅ **Educational & Research Purposes Only**: This tool is designed for technical analysis and documentation
- ✅ **No Copyrighted Content**: No game assets, artwork, or proprietary code are included or distributed
- ✅ **Technical Focus**: All documentation focuses on file format specifications, data structures, and encoding information
- ✅ **Nominal Fair Use**: References to "Dispel" are for identification and compatibility purposes only
- ✅ **User Responsibility**: Users must own a legitimate copy of the game for testing and comply with all applicable laws

### Prohibited Use

This tool must **NOT** be used for:

- Distributing copyrighted game content
- Creating derivative works that infringe on intellectual property
- Commercial exploitation of game assets
- Bypassing copy protection mechanisms
- Any illegal or unethical purposes

---

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

---

## 📦 Project Structure

```
dispel-extractor/
├── Cargo.toml                    # Rust project configuration
├── Makefile                      # Common build and analysis tasks
├── src/
│   ├── main.rs                   # CLI entry point and command dispatcher
│   ├── lib.rs                    # Core library with data structures
│   ├── database.rs               # SQLite database interaction
│   ├── map/                      # Map file parsing and rendering
│   │   ├── mod.rs                # Map module exports
│   │   ├── reader.rs             # Binary map file reader
│   │   ├── render.rs             # Map rendering logic
│   │   ├── tileset.rs            # Tile extraction and plotting
│   │   ├── sprite_loader.rs      # Sprite loading utilities
│   │   ├── model.rs              # Map data structures
│   │   └── types.rs              # Type definitions
│   ├── sprite.rs                 # Sprite and animation extraction
│   ├── snf.rs                    # SNF audio file conversion
│   └── commands/                 # CLI command implementations
│       ├── map.rs                # Map-related commands
│       ├── sprite.rs             # Sprite-related commands
│       ├── sound.rs              # Sound conversion commands
│       ├── database.rs           # Database import commands
│       └── refs.rs               # Reference file parsing commands
├── dispel-gui/                   # Optional GUI frontend (workspace member)
├── fixtures/                     # Sample game files for testing
├── database.sqlite               # Extracted game data database
├── docs/                         # Technical documentation
│   ├── overview.md
│   ├── file_formats.md
│   ├── database_and_references.md
│   └── files/                    # Detailed file format specifications
└── target/                       # Build artifacts
```

---

## 🚀 Installation & Usage

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)
- Git

### Build Instructions

```bash
# Clone the repository
git clone .../dispel-extractor.git

# Build the tool
cd dispel-extractor

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
./target/release/dispel-extractor map --help
./target/release/dispel-extractor ref --help
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

### Import Game Data to SQLite

```bash
# Import all supported game data into database.sqlite
./target/release/dispel-extractor database import
```

### Parse Reference Files

```bash
# Parse a specific reference file and export as JSON
./target/release/dispel-extractor ref map fixtures/Dispel/Ref/Map.ini

# Parse all supported reference types
./target/release/dispel-extractor ref all-maps fixtures/Dispel/AllMap.ini
./target/release/dispel-extractor ref weapons fixtures/Dispel/CharacterInGame/weaponItem.db
./target/release/dispel-extractor ref monsters fixtures/Dispel/MonsterInGame/Monster.db
./target/release/dispel-extractor ref heal-items fixtures/Dispel/CharacterInGame/HealItem.db
```

---

## 📊 Supported File Types

| File Type | Description | Supported |
|-----------|-------------|-----------|
| `.map` | Map definition files | ✅ |
| `.btl` | Background tile layer files | ✅ |
| `.gtl` | Ground tile layer files | ✅ |
| `.SPR` | Sprite and animation files | ✅ |
| `.snf` | Sound effect files | ✅ |
| `.ini` | Configuration files | ✅ |
| `.db` | Database files (items, monsters, etc.) | ✅ |
| `.ref` | Reference files | ✅ |
| `.dlg` | Dialogue files | ✅ |
| `.scr` | Script files | ✅ |

---

## 📚 Documentation

Detailed technical documentation is available in the [`docs/`](docs/) directory:

- **[overview.md](docs/overview.md)**: High-level overview of the project structure and workflow
- **[file_formats.md](docs/file_formats.md)**: Detailed specifications of file formats
- **[database_and_references.md](docs/database_and_references.md)**: Database schema and reference file formats
- **[rendering.md](docs/rendering.md)**: Map rendering techniques and algorithms
- **[files/](docs/files/)**: Individual file format specifications

---

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

### What to Contribute

- Improve file format documentation
- Add support for new file types
- Enhance error handling and validation
- Improve code quality and safety
- Add unit tests
- Update technical specifications

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes
4. Test thoroughly
5. Commit with clear messages
6. Push to your fork
7. Open a pull request

### Code Style

- Follow Rust community style guidelines
- Use `rustfmt` for formatting
- Include comments for complex logic
- Write clear, concise commit messages

### Legal Requirements

All contributions must:
- Maintain compliance with the legal disclaimer
- Focus on technical analysis, not content distribution
- Not include any copyrighted material
- Respect trademark usage guidelines

---

## 📜 License

This research tool is licensed under the **[MIT License](LICENSE)** for the **code only**.

### License Scope

✅ **Applies to**: All source code in this repository
❌ **Does not apply to**: Game content, assets, or proprietary formats

### Attribution

```
Copyright (c) 2024 Dispel Extractor Contributors

Permission is hereby granted... (standard MIT license text)
```

---

## 📧 Contact & Support

For legal inquiries, compliance questions, or general support:

1. **Open an Issue**: [GitHub Issues](https://github.com/your-org/dispel-extractor/issues)
2. **Review Documentation**: Check the [`docs/`](docs/) directory for technical details
3. **Check Compliance**: Review the [Legal Compliance](#%EF%B8%8F-legal-compliance) section

---

## 🙏 Acknowledgments

- The DISPEL® game developers for creating the original game
- The reverse engineering community for inspiration and guidance
- Rust open-source community for excellent tools and libraries

---

## 📈 Project Status

- ✅ Core map rendering and extraction
- ✅ Sprite and animation extraction
- ✅ Sound conversion
- ✅ Database import and reference parsing
- ✅ Extensive technical documentation
- 🚧 GUI frontend (in development)
- 🚧 Additional file format support
- 🚧 Performance optimization

---

🔒 **This project operates under fair use principles for reverse engineering research and educational purposes.**
