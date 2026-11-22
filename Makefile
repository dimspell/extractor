# Dispel Extractor Makefile
# Use `make <target>` to run common tasks.
# Pass variable overrides for parameterized targets:
#   make extract-file FILE=fixtures/Dispel/AllMap.ini
#   make map-render map_id=cat1
#
# Default paths (overridable):
#   game_path  = fixtures/Dispel
#   map_id     = cat1
#   db_path    = database.sqlite
#   out        = out

# Single source of truth for the map-render flavors. The generated recipe below
# and .PHONY both derive from this list; adding a flavor touches one place only.
MAP_RENDER_TARGETS := map-render map-render-legacy map-render-lit \
                      map-render-transparent map-render-collisions map-render-events \
                      map-render-draw-items map-render-waypoints \
                      map-render-noground map-render-nobuildings map-render-noroofs \
                      map-render-nomonsters map-render-nonpcs map-render-noobjects

# The Rust `test_makefile_invocations_*` tests in src/cli.rs validate every
# `cargo run -- <cmd>` invocation below against the declarative command spec.
# scripts/check-project-metadata.sh verifies .PHONY matches the real recipes and
# that every fmt/clippy/test command routes through scripts/ci-commands.sh.
.PHONY: help command-list fmt cargo_test iced_test hexedit_test clippy check build run \
        run-custom-context-menu hexedit check_iced_config check_metadata check_metadata_test \
        extract-help extracts-help extract-file extract-file-pretty \
        extract-all-maps extract-map extract-monsters extract-heal-item \
        extract-draw-item extract-misc-item extract-edit-item \
        extract-event-item extract-store-db extract-extra-ref \
        extract-monster-ref extract-npc-ref extract-multi-magic \
        extract-party-level extract-event-npc-ref extract-party-ini \
        extract-quest extract-message extract-ch-data extract-dialog \
        extract-dialogue-text extract-weapons \
        patch-dry-run patch-in-place patch validate validate-verbose \
        list list-json list-filtered schema template template-pretty \
        map-help map-tiles map-atlas $(MAP_RENDER_TARGETS) \
        map-extract-sprites map-to-json map-to-json-pretty map-to-db \
        map-atlas-btl map-atlas-gtl \
        map-all-gtl-tiles map-all-btl-tiles zip-map-tiles \
        database-import database-dialog-texts database-maps \
        database-databases database-refs database-rest \
        sprite sprite-animation sprite-animation-legacy sprite-sprite sprite-info \
        sound sound-legacy dialog dialog-basic dialogs \
        mod-pack mod-pack-pretty mod-pack-single cli-smoke

# ============================================================================
# Configuration variables (overridable via `make TARGET var=val`)
# ============================================================================

game_path ?= fixtures/Dispel
map_id    ?= cat1
db_path   ?= database.sqlite
out       ?= out
sprite_path ?= $(game_path)/CharacterInGame/M_BODY1.SPR

# ============================================================================
# Utility targets
# ============================================================================

help:
	cargo run -- --help

# Machine-friendly command list generated from the declarative spec in src/cli.rs.
command-list:
	cargo run -- --list-commands

fmt:
	bash scripts/ci-commands.sh fmt

cargo_test:
	bash scripts/ci-commands.sh test

iced_test:
	bash scripts/ci-commands.sh iced-test

hexedit_test:
	bash scripts/ci-commands.sh hexedit-test

# Build, fmt, clippy and test commands live in scripts/ci-commands.sh so local
# `make` and GitHub Actions cannot silently diverge.
clippy:
	bash scripts/ci-commands.sh clippy

check:
	bash scripts/ci-commands.sh check

# Verify the forked-iced pin set / feature matrix / test backend are coherent.
check_iced_config:
	bash scripts/ci-commands.sh check-iced-config

# Verify AGENTS.md counts, .PHONY parity, CI command parity and the shared matrix.
check_metadata:
	bash scripts/ci-commands.sh check-metadata

# Self-test the validator by injecting drift into a scratch copy of the tree.
check_metadata_test:
	bash scripts/ci-commands.sh check-metadata-test

build:
	bash scripts/ci-commands.sh build

run:
	cargo run -p dispel-gui

run-custom-context-menu:
	sh -ac "DISPEL_FORCE_CUSTOM_CONTEXT_MENU=1 cargo run -p dispel-gui"

hexedit:
	cargo run -p hexedit -- file.bin --script-dir ./scripts

# ============================================================================
# extract — Read game files and output as JSON
# ============================================================================
# Flags: -i/--input <PATH>   Game file to read
#        -o/--output <PATH>  Output path (default: stdout)
#        -t/--type <TYPE>    File type override (auto-detected from filename)
#        -p/--pretty         Pretty-print JSON

extract-help:
	cargo run -- extract --help

extract-file:
	cargo run -- extract -i "$(FILE)" -o "$(out)/$(notdir $(FILE)).json"

extract-file-pretty:
	cargo run -- extract -i "$(FILE)" -o "$(out)/$(notdir $(FILE)).json" --pretty

# --- Batch: all known game files ---

extracts-help:
	cargo run -- extract --help
	cargo run -- extract -i "$(game_path)/AllMap.ini"
	cargo run -- extract -i "$(game_path)/Ref/Map.ini"
	cargo run -- extract -i "$(game_path)/Extra.ini"
	cargo run -- extract -i "$(game_path)/Event.ini"
	cargo run -- extract -i "$(game_path)/Monster.ini"
	cargo run -- extract -i "$(game_path)/Npc.ini"
	cargo run -- extract -i "$(game_path)/Wave.ini"
	cargo run -- extract -i "$(game_path)/Ref/PartyRef.ref"
	cargo run -- extract -i "$(game_path)/Ref/DRAWITEM.ref"
	cargo run -- extract -i "$(game_path)/NpcInGame/Dlgcat1.dlg"
	cargo run -- extract -i "$(game_path)/NpcInGame/PartyPgp.pgp"
	cargo run -- extract -i "$(game_path)/CharacterInGame/weaponItem.db"
	cargo run -- extract -i "$(game_path)/MonsterInGame/Monster.db"
	cargo run -- extract -i "$(game_path)/CharacterInGame/STORE.DB"
	cargo run -- extract -i "$(game_path)/NpcInGame/Npccat1.ref"
	cargo run -- extract -i "$(game_path)/MonsterInGame/Mondun01.ref"
	cargo run -- extract -i "$(game_path)/CharacterInGame/MiscItem.db"
	cargo run -- extract -i "$(game_path)/CharacterInGame/HealItem.db"
	cargo run -- extract -i "$(game_path)/ExtraInGame/Extdun01.ref"
	cargo run -- extract -i "$(game_path)/CharacterInGame/EventItem.db"
	cargo run -- extract -i "$(game_path)/CharacterInGame/EditItem.db"
	cargo run -- extract -i "$(game_path)/NpcInGame/PrtLevel.db"
	cargo run -- extract -i "$(game_path)/NpcInGame/Eventnpc.ref"

# --- Individual file extracts (convenience shorthands) ---

extract-all-maps:
	cargo run -- extract -i "$(game_path)/AllMap.ini"

extract-map:
	cargo run -- extract -i "$(game_path)/Ref/Map.ini"

extract-monsters:
	cargo run -- extract -i "$(game_path)/MonsterInGame/Monster.db"

extract-heal-item:
	cargo run -- extract -i "$(game_path)/CharacterInGame/HealItem.db"

extract-draw-item:
	cargo run -- extract -i "$(game_path)/Ref/DRAWITEM.ref"

extract-misc-item:
	cargo run -- extract -i "$(game_path)/CharacterInGame/MiscItem.db"

extract-edit-item:
	cargo run -- extract -i "$(game_path)/CharacterInGame/EditItem.db"

extract-event-item:
	cargo run -- extract -i "$(game_path)/CharacterInGame/EventItem.db"

extract-store-db:
	cargo run -- extract -i "$(game_path)/CharacterInGame/STORE.DB"

extract-extra-ref:
	cargo run -- extract -i "$(game_path)/ExtraInGame/Extdun01.ref"

extract-monster-ref:
	cargo run -- extract -i "$(game_path)/MonsterInGame/Mondun01.ref"

extract-npc-ref:
	cargo run -- extract -i "$(game_path)/NpcInGame/Npccat1.ref"

extract-multi-magic:
	cargo run -- extract -i "$(game_path)/MagicInGame/MulMagic.db"

extract-party-level:
	cargo run -- extract -i "$(game_path)/NpcInGame/PrtLevel.db"

extract-event-npc-ref:
	cargo run -- extract -i "$(game_path)/NpcInGame/Eventnpc.ref"

extract-party-ini:
	cargo run -- extract -i "$(game_path)/NpcInGame/PrtIni.db"

extract-quest:
	cargo run -- extract -i "$(game_path)/ExtraInGame/Quest.scr"

extract-message:
	cargo run -- extract -i "$(game_path)/ExtraInGame/Message.scr"

extract-ch-data:
	cargo run -- extract -i "$(game_path)/CharacterInGame/ChData.db"

extract-dialog:
	cargo run -- extract -i "$(game_path)/NpcInGame/Dlgcat1.dlg"

extract-dialogue-text:
	cargo run -- extract -i "$(game_path)/NpcInGame/PartyPgp.pgp"

extract-weapons:
	cargo run -- extract -i "$(game_path)/CharacterInGame/weaponItem.db"

# ============================================================================
# patch — Write JSON data back to game binary files
# ============================================================================
# Flags: -i/--input  <PATH>   Source JSON file
#        -t/--target <PATH>   Game file to patch
#        -o/--output <PATH>   Output path (default: same as target)
#        --type <TYPE>        File type override
#        -d/--dry-run         Validate without writing
#        --in-place           Patch target directly (creates .bak backup)
#        --no-backup          Skip backup creation (with --in-place)

patch-dry-run:
	cargo run -- patch -i "$(INPUT)" -t "$(TARGET)" --dry-run

patch-in-place:
	cargo run -- patch -i "$(INPUT)" -t "$(TARGET)" --in-place

patch:
	cargo run -- patch -i "$(INPUT)" -t "$(TARGET)" -o "$(OUTPUT)"

# ============================================================================
# validate — Validate JSON against a file format schema
# ============================================================================
# Flags: -i/--input <PATH>  JSON file to validate
#        --type <TYPE>      File type (required)
#        --verbose          Detailed error output

validate:
	cargo run -- validate -i "$(INPUT)" --type "$(TYPE)"

validate-verbose:
	cargo run -- validate -i "$(INPUT)" --type "$(TYPE)" --verbose

# ============================================================================
# list — Show supported file types
# ============================================================================
# Flags: --format <text|json>  Output format (default: text)
#        --filter <PATTERN>    Filter by name/description

list:
	cargo run -- list

list-json:
	cargo run -- list --format json

list-filtered:
	cargo run -- list --filter "$(FILTER)"

# ============================================================================
# schema — Generate JSON Schema for a file type
# ============================================================================
# Flags: --type <TYPE>  File type (required)

schema:
	cargo run -- schema --type "$(TYPE)"

# ============================================================================
# template — Generate a minimal JSON template for a file type
# ============================================================================
# Flags: --type <TYPE>  File type (required)
#        -p/--pretty    Pretty-print JSON

template:
	cargo run -- template --type "$(TYPE)"

template-pretty:
	cargo run -- template --type "$(TYPE)" --pretty

# ============================================================================
# map — Map extraction, rendering, and conversion
# ============================================================================
# Subcommands:
#   map tiles   – Extract every tile as a separate image
#   map atlas   – Pack tiles into a single atlas PNG
#   map render  – Render map with layers, overlays, and sprites
#   map to-db   – Import .MAP file into SQLite
#   map sprites – Extract map-internal sprites to PNGs
#   map to-json – Export map data as JSON

map-help:
	cargo run -- map --help

# map tiles <INPUT> [--output <DIR>]
map-tiles:
	cargo run -- map tiles "$(INPUT)" --output "$(OUTPUT)"

# map atlas <INPUT> <OUTPUT>
map-atlas:
	cargo run -- map atlas "$(INPUT)" "$(OUTPUT)"

# map render --map <.map> --btl <.btl> --gtl <.gtl> --output <.png>
# Optional flags:
#   --game-path <DIR>      Enable entity overlay (NPCs, monsters, extras)
#   --save-sprites         Export sub-sprites from the map file
#   --transparent          RGBA PNG with alpha channel
#   --collisions           Show collision overlay
#   --events               Show event overlay
#   --draw-items           Show draw item overlay
#   --npc-waypoints        Show NPC waypoint arrows
#   --no-ground            Hide ground (GTL) layer
#   --no-buildings         Hide buildings (BTL objects) layer
#   --no-roofs             Hide roof (BTL tile) layer
#   --no-internal-sprites  Hide embedded map sprites
#   --no-monsters          Hide external monster rendering
#   --no-npcs              Hide external NPC rendering
#   --no-objects           Hide external objects (extras) rendering

# One canonical argument list, one per-flavor flag table, one recipe. Every
# name in $(MAP_RENDER_TARGETS) keeps working; a common flag change touches
# MAP_RENDER_ARGS only. Add a flavor by appending it to MAP_RENDER_TARGETS and
# (if needed) giving it a MAP_RENDER_EXTRA value below.
MAP_RENDER_ARGS = --map="$(game_path)/Map/$(map_id).map" \
                  --btl="$(game_path)/Map/$(map_id).btl" \
                  --gtl="$(game_path)/Map/$(map_id).gtl" \
                  --output="map.png" \
                  --game-path="$(game_path)"

# Render with the game's viewport light template (500x1 PNG). The renderer
# ignores the .map file's own light bits and takes one light level per visible
# slot from the template instead, so dark maps need it to render lit.
#   make map-render-lit map_id=dun01 camera=124,124
# The asset location is configurable; no sibling-repo checkout is assumed.
#   make map-render-lit light_template=/path/to/light_template.png
light_template ?= $(game_path)/Map/light_template.png
camera ?=

# Flavor flag table (empty for map-render and its legacy alias).
map-render-lit: MAP_RENDER_EXTRA = --light-template="$(light_template)" $(if $(camera),--camera="$(camera)",)
map-render-transparent: MAP_RENDER_EXTRA = --transparent
map-render-collisions: MAP_RENDER_EXTRA = --collisions
map-render-events: MAP_RENDER_EXTRA = --events
map-render-draw-items: MAP_RENDER_EXTRA = --draw-items
map-render-waypoints: MAP_RENDER_EXTRA = --npc-waypoints
map-render-noground: MAP_RENDER_EXTRA = --no-ground
map-render-nobuildings: MAP_RENDER_EXTRA = --no-buildings
map-render-noroofs: MAP_RENDER_EXTRA = --no-roofs
map-render-nomonsters: MAP_RENDER_EXTRA = --no-monsters
map-render-nonpcs: MAP_RENDER_EXTRA = --no-npcs
map-render-noobjects: MAP_RENDER_EXTRA = --no-objects

$(MAP_RENDER_TARGETS):
	cargo run -- map render $(MAP_RENDER_ARGS) $(MAP_RENDER_EXTRA)

# map sprites <INPUT> [--output <DIR>]
map-extract-sprites:
	cargo run -- map sprites \
		"$(game_path)/Map/$(map_id).map" \
		--output "$(out)/$(map_id)_sprites"

# map to-json --input <.map> [--output <JSON>] [--pretty]
map-to-json:
	cargo run -- map to-json --input "$(INPUT)" --output "$(OUTPUT)"

map-to-json-pretty:
	cargo run -- map to-json --input "$(INPUT)" --output "$(OUTPUT)" --pretty

# map to-db --database <DB> --map <.map>
map-to-db:
	cargo run -- map to-db --database "$(db_path)" --map "$(INPUT)"

# map atlas convenience (shortcut: make atlas from game path + map_id)
map-atlas-btl:
	cargo run -- map atlas \
		"$(game_path)/Map/$(map_id).btl" \
		"$(map_id).btl.png"

map-atlas-gtl:
	cargo run -- map atlas \
		"$(game_path)/Map/$(map_id).gtl" \
		"$(map_id).gtl.png"

# Batch tile processing across all maps
MAP_NAMES ?= $(foreach f,$(filter-out fixtures/Dispel/Map/map4.map,$(wildcard fixtures/Dispel/Map/*.map)),$(basename $(notdir $(f))))

map-all-gtl-tiles:
	for map in $(MAP_NAMES); do cargo run -- map tiles "$(game_path)/Map/$$map.gtl" --output "$(out)/$$map-gtl"; done

map-all-btl-tiles:
	for map in $(MAP_NAMES); do cargo run -- map tiles "$(game_path)/Map/$$map.btl" --output "$(out)/$$map-btl"; done

zip-map-tiles:
	for map in $(MAP_NAMES); do zip -r "$(out)/$$map.zip" "$(out)/$$map-gtl/" "$(out)/$$map-btl/"; done

# ============================================================================
# database — SQLite import commands
# ============================================================================
# Subcommands:
#   import         Import all (full pipeline)
#   dialog-texts   Dialog and PGP texts only
#   maps           Map files only
#   databases      .db item/character files only
#   refs           INI config files only
#   rest           REF/PGP files only

database-import:
	cargo run -- database import "$(game_path)" "$(db_path)"

database-dialog-texts:
	cargo run -- database dialog-texts "$(game_path)" "$(db_path)"

database-maps:
	cargo run -- database maps "$(game_path)" "$(db_path)"

database-databases:
	cargo run -- database databases "$(game_path)" "$(db_path)"

database-refs:
	cargo run -- database refs "$(game_path)" "$(db_path)"

database-rest:
	cargo run -- database rest "$(game_path)" "$(db_path)"

# ============================================================================
# sprite — Extract frames or animations from SPR files
# ============================================================================
# Usage: sprite <INPUT> [--mode <sprite|animation>] [--info]

sprite:
	cargo run -- sprite "$(INPUT)"

sprite-animation:
	cargo run -- sprite "$(INPUT)" --mode animation

sprite-info:
	cargo run -- sprite "$(INPUT)" --info

# (Legacy convenience: extract from the default sprite file)
sprite-sprite:
	cargo run -- sprite "$(sprite_path)" --mode sprite

sprite-animation-legacy:
	cargo run -- sprite "$(sprite_path)" --mode animation

# ============================================================================
# sound — Convert SNF audio to WAV
# ============================================================================
# Usage: make sound INPUT=<file.snf> OUTPUT=<file.wav>
# The CLI requires an explicit direction: `sound to-wav|from-wav <IN> <OUT>`.

sound:
	cargo run -- sound to-wav "$(INPUT)" "$(OUTPUT)"

# (Legacy convenience)
sound-legacy:
	cargo run -- sound to-wav notexist.snf Piach.wav

# ============================================================================
# dialog — Print dialog flow from DLG/PGP files
# ============================================================================
# Usage: dialog <DLG_FILE> [-p <PGP>] [-n <NPC_REF>] [-d <DATABASE>]
# Variables: DLG, PGP, NPC_REF, DB

dialog:
	cargo run -- dialog "$(DLG)" -p "$(PGP)" -n "$(NPC_REF)" -d "$(DB)"

dialog-basic:
	cargo run -- dialog "$(DLG)"

# (Legacy full pipeline)
dialogs:
	cargo run -- dialog "$(game_path)/NpcInGame/Dlgcat1.dlg" \
		-p "$(game_path)/NpcInGame/Pgpcat1.pgp" \
		-n "$(game_path)/NpcInGame/Npccat1.ref" \
		--database "$(db_path)"

# ============================================================================
# mod-pack — Export event scripts to JSON (for Godot etc.)
# ============================================================================
# Flags: -g/--game-path <DIR>   Game directory (contains Ref/Event*.scr)
#        -o/--output <DIR>      Output directory (default: mod-pack)
#        -p/--pretty            Pretty-print JSON
#        -s/--single-file       Single JSON array instead of per-file

mod-pack:
	cargo run -- mod-pack -g "$(game_path)" -o "$(OUTPUT)"

mod-pack-pretty:
	cargo run -- mod-pack -g "$(game_path)" -o "$(OUTPUT)" --pretty

mod-pack-single:
	cargo run -- mod-pack -g "$(game_path)" -o "$(OUTPUT)" --single-file

# ============================================================================
# cli-smoke — CLI smoke test (uses a real registered subcommand)
# ============================================================================

cli-smoke:
	cargo run -- list --format json
