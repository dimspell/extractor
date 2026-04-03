map_id ?= cat1

sound:
	cargo run -- sound --input notexist.snf --output Piach.wav


maps ?= $(wildcard fixtures/Dispel/Map/*.map)
maps_except_map4 ?= $(filter-out fixtures/Dispel/Map/map4.map,$(maps))
map_names ?= $(foreach map,$(maps_except_map4),$(basename $(notdir $(map))))

sprite_path ?= fixtures/Dispel/CharacterInGame/M_BODY1.SPR
sprites_wildcard ?= $(wildcard fixtures/Dispel/CharacterInGame/*.SPR)

# replace_map_with_btl_extension=$(subst .map,.btl,$$map); \
# replace_map_with_gtl_extension=$(subst .map,.gtl,$$map); \

# 		echo run -- map render \
# 			--map="$$map" \
# 			--btl="fixtures/Dispel/Map/$$map.btl" \
# 			--gtl="fixtures/Dispel/Map/$$map.gtl" \
# 			--output="out/$$map.png" \

# This command iterates overall the maps and renders all the tiles to a separate file for each map (uses the map tiles --output "{map_name}_(gtl|btl).png").
map-all-gtl-tiles:
	for map in $(map_names); do cargo run -- map tiles "fixtures/Dispel/Map/$$map.gtl" --output "out/$$map-gtl"; done

map-all-btl-tiles:
	for map in $(map_names); do cargo run -- map tiles "fixtures/Dispel/Map/$$map.btl" --output "out/$$map-btl"; done

zip-al-subdirectories:
# 	$(foreach dir,$(map_names),zip -r "out/$$dir.zip" "out/$$dir-gtl/" "out/$$dir-btl/")
	for dir in $(map_names); do zip -r "out/$$dir.zip" "out/$$dir-gtl/" "out/$$dir-btl/"; done

sprite-sprite:
	# cargo run -- sprite --input="$(sprite_path)" --mode=sprite
	./target/debug/dispel-extractor sprite "$(sprite_path)" --mode=sprite

sprite-animation:
	# cargo run -- sprite --input="$(sprite_path)" --mode=animation
	./target/debug/dispel-extractor sprite "$(sprite_path)" --mode=animation

map-sprites:
	# cargo run -- map sprites --input="$(sprite_path)"
	./target/debug/dispel-extractor map sprites "fixtures/Dispel/Map/${map_id}.map"

map-atlas-btl:
	cargo run -- map atlas \
		"fixtures/Dispel/Map/${map_id}.btl" \
		"${map_id}.btl.png"

map-atlas-gtl:
	cargo run -- map atlas \
		"fixtures/Dispel/Map/${map_id}.gtl" \
		"${map_id}.gtl.png"

map-render:
	cargo run -- map render \
		--map="fixtures/Dispel/Map/${map_id}.map" \
		--btl="fixtures/Dispel/Map/${map_id}.btl" \
		--gtl="fixtures/Dispel/Map/${map_id}.gtl" \
		--output="map.png"

map-extract-sprites:
	cargo run -- map sprites \
		"fixtures/Dispel/Map/${map_id}.map" \
		--output "out/${map_id}_sprites"

map-from-db:
	cargo run -- map from-db --map-id ${map_id} --gtl-atlas ${map_id}.gtl.png --btl-atlas ${map_id}.btl.png -o out_${map_id}.png

map-from-db-with-sprite:
	cargo run -- map from-db --map-id ${map_id} --gtl-atlas ${map_id}.gtl.png --btl-atlas ${map_id}.btl.png -o out_${map_id}.png --game-path "fixtures/Dispel"

ref-all-maps:
	cargo run -- ref all-maps "fixtures/Dispel/AllMap.ini"

ref-map:
	cargo run -- ref map &Path::new("sample-data/Ref/Map.ini")

ref-monsters:
	cargo run -- ref monsters "fixtures/Dispel/MonsterInGame/Monster.db"

ref-heal-item:
	cargo run -- ref heal-items "fixtures/Dispel/CharacterInGame/HealItem.db"

ref-draw-item:
	cargo run -- ref draw-item "fixtures/Dispel/Ref/DRAWITEM.ref"

ref-misc-item:
	cargo run -- ref misc-item "fixtures/Dispel/CharacterInGame/MiscItem.db"

ref-edit-item:
	cargo run -- ref edit-items "fixtures/Dispel/CharacterInGame/EditItem.db"

ref-event-item:
	cargo run -- ref event-items "fixtures/Dispel/CharacterInGame/EventItem.db"

ref-store-db:
	cargo run -- ref store "fixtures/Dispel/CharacterInGame/STORE.DB"

ref-extra-ref:
	cargo run -- ref extra-ref "fixtures/Dispel/ExtraInGame/Extdun01.ref"

ref-monster-ref:
	cargo run -- ref monster-ref "fixtures/Dispel/MonsterInGame/Mondun01.ref"

ref-npc-ref:
	cargo run -- ref npc-ref "fixtures/Dispel/NpcInGame/Npccat1.ref"

ref-multi-magic:
	cargo run -- ref multi-magic "fixtures/Dispel/MagicInGame/MulMagic.db"

ref-party-level:
	cargo run -- ref party-level "fixtures/Dispel/NpcInGame/PrtLevel.db"

ref-event-npc-ref:
	cargo run -- ref event-npc-ref "fixtures/Dispel/NpcInGame/Eventnpc.ref"

ref-party-ini:
	cargo run -- ref party-ini fixtures/Dispel/NpcInGame/PrtIni.db

ref-quest:
	cargo run -- ref quest "fixtures/Dispel/ExtraInGame/Quest.scr"

ref-message:
	cargo run -- ref message "fixtures/Dispel/ExtraInGame/Message.scr"

ref-ch-data:
	cargo run -- ref ch-data "fixtures/Dispel/CharacterInGame/ChData.db"

ref-dialog:
	cargo run -- ref dialog "fixtures/Dispel/NpcInGame/Dlgcat1.dlg"

ref-dialogue-text:
	cargo run -- ref dialog-texts "fixtures/Dispel/NpcInGame/PartyPgp.pgp"

ref-weapons:
	cargo run -- ref weapons "fixtures/Dispel/CharacterInGame/weaponItem.db"

database-import:
	cargo run -- database import "fixtures/Dispel" "database.sqlite"

refs-help:
	cargo run -- ref --help
	cargo run -- ref all-maps "fixtures/Dispel/AllMap.ini"
	cargo run -- ref map "fixtures/Dispel/Ref/Map.ini"
	cargo run -- ref extra "fixtures/Dispel/Extra.ini"
	cargo run -- ref event "fixtures/Dispel/Event.ini"
	cargo run -- ref monster "fixtures/Dispel/Monster.ini"
	cargo run -- ref npc "fixtures/Dispel/Npc.ini"
	cargo run -- ref wave "fixtures/Dispel/Wave.ini"
	cargo run -- ref party-ref "fixtures/Dispel/Ref/PartyRef.ref"
	cargo run -- ref draw-item "fixtures/Dispel/Ref/DRAWITEM.ref"
	cargo run -- ref dialog "fixtures/Dispel/NpcInGame/Dlgcat1.dlg"
	cargo run -- ref dialog-texts "fixtures/Dispel/NpcInGame/PartyPgp.pgp"
	cargo run -- ref weapons "fixtures/Dispel/CharacterInGame/weaponItem.db"
	cargo run -- ref monsters "fixtures/Dispel/MonsterInGame/Monster.db"
	# cargo run -- ref multi-magic "fixtures/Dispel/MagicInGame/MulMagic.db"
	cargo run -- ref store "fixtures/Dispel/CharacterInGame/STORE.DB"
	cargo run -- ref npc-ref "fixtures/Dispel/NpcInGame/Npccat1.ref"
	cargo run -- ref monster-ref "fixtures/Dispel/MonsterInGame/Mondun01.ref"
	cargo run -- ref misc-item "fixtures/Dispel/CharacterInGame/MiscItem.db"
	cargo run -- ref heal-items "fixtures/Dispel/CharacterInGame/HealItem.db"
	cargo run -- ref extra-ref "fixtures/Dispel/ExtraInGame/Extdun01.ref"
	cargo run -- ref event-items "fixtures/Dispel/CharacterInGame/EventItem.db"
	cargo run -- ref edit-items "fixtures/Dispel/CharacterInGame/EditItem.db"
	cargo run -- ref party-level "fixtures/Dispel/NpcInGame/PrtLevel.db"
	cargo run -- ref event-npc-ref "fixtures/Dispel/NpcInGame/Eventnpc.ref"

help:
	cargo run -- --help
