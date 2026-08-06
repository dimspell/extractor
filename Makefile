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

.PHONY: help fmt cargo_test iced_test clippy run run-custom-context-menu \
        check build hexedit \
        extract-help extract-file extract-file-pretty \
        extract-all-maps extract-map extract-monsters extract-heal-item \
        extract-draw-item extract-misc-item extract-edit-item \
        extract-event-item extract-store-db extract-extra-ref \
        extract-monster-ref extract-npc-ref extract-multi-magic \
        extract-party-level extract-event-npc-ref extract-party-ini \
        extract-quest extract-message extract-ch-data extract-dialog \
        extract-dialogue-text extract-weapons \
        patch-dry-run patch-in-place patch validate validate-verbose \
        list list-json list-filtered schema template template-pretty \
        map-help map-tiles map-atlas map-render map-render-full \
        map-render-transparent map-render-collisions map-render-events \
        map-render-draw-items map-render-waypoints \
        map-render-noground map-render-nobuildings map-render-noroofs \
        map-render-nomonsters map-render-nonpcs map-render-noobjects \
        map-extract-sprites map-to-json map-to-json-pretty map-to-db \
        map-from-db map-from-db-with-sprite \
        map-atlas-btl map-atlas-gtl \
        map-all-gtl-tiles map-all-btl-tiles zip-map-tiles \
        database-import database-dialog-texts database-maps \
        database-databases database-refs database-rest \
        sprite sprite-animation sprite-info \
        sound dialog dialog-basic dialogs \
        mod-pack mod-pack-pretty mod-pack-single \
        test-hello

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

fmt:
	cargo fmt --all

cargo_test:
	cargo test --workspace --all-features --quiet

iced_test:
	cargo test -p dispel-gui --features "iced_test app::tests"

clippy:
	cargo clippy --workspace -- -D warnings

check:
	cargo check --workspace --message-format=short

build:
	cargo build --workspace

run:
	cargo run -p dispel-gui

run-custom-context-menu:
	sh -ac "FORCE_CUSTOM_CONTEXT_MENU=1 cargo run -p dispel-gui"

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
#   map from-db – Render map from SQLite + atlas PNGs
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
#   --full-map             Render full canvas (no occlusion viewport)
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

map-render:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)"

map-render-full:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--full-map

map-render-transparent:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--transparent

map-render-collisions:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--collisions

map-render-events:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--events

map-render-draw-items:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--draw-items

map-render-waypoints:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--npc-waypoints

# Layer visibility toggles
map-render-noground:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--no-ground

map-render-nobuildings:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--no-buildings

map-render-noroofs:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--no-roofs

map-render-nomonsters:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--no-monsters

map-render-nonpcs:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--no-npcs

map-render-noobjects:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)" \
		--no-objects

# (Legacy alias: maps that used the old direct CLI style)
map-render-legacy:
	cargo run -- map render \
		--map="$(game_path)/Map/$(map_id).map" \
		--btl="$(game_path)/Map/$(map_id).btl" \
		--gtl="$(game_path)/Map/$(map_id).gtl" \
		--output="map.png" \
		--game-path="$(game_path)"

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

# map from-db --database <DB> --map-id <ID> --gtl-atlas <PNG> --btl-atlas <PNG> --output <PNG>
#                  [--game-path <DIR>] [--atlas-columns <NUM>]
map-from-db:
	cargo run -- map from-db \
		--database "$(db_path)" \
		--map-id "$(map_id)" \
		--gtl-atlas "$(map_id).gtl.png" \
		--btl-atlas "$(map_id).btl.png" \
		-o "$(out)_$(map_id).png"

map-from-db-with-sprite:
	cargo run -- map from-db \
		--database "$(db_path)" \
		--map-id "$(map_id)" \
		--gtl-atlas "$(map_id).gtl.png" \
		--btl-atlas "$(map_id).btl.png" \
		-o "$(out)_$(map_id).png" \
		--game-path "$(game_path)"

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
# Usage: sound <INPUT> <OUTPUT>

sound:
	cargo run -- sound "$(INPUT)" "$(OUTPUT)"

# (Legacy convenience)
sound-legacy:
	cargo run -- sound --input notexist.snf --output Piach.wav

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
		--database-path "$(db_path)"

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
# test — CLI smoke test
# ============================================================================
# Usage: test [-m/--message <TEXT>] (default: "Hello from test command!")

test-hello:
	cargo run -- test -m "$(MESSAGE)"
