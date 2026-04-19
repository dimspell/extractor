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
		--output="map.png" \
		--game-path="fixtures/Dispel"

map-extract-sprites:
	cargo run -- map sprites \
		"fixtures/Dispel/Map/${map_id}.map" \
		--output "out/${map_id}_sprites"

map-from-db:
	cargo run -- map from-db --map-id ${map_id} --gtl-atlas ${map_id}.gtl.png --btl-atlas ${map_id}.btl.png -o out_${map_id}.png

map-from-db-with-sprite:
	cargo run -- map from-db --map-id ${map_id} --gtl-atlas ${map_id}.gtl.png --btl-atlas ${map_id}.btl.png -o out_${map_id}.png --game-path "fixtures/Dispel"

extract-all-maps:
	cargo run -- extract -i "fixtures/Dispel/AllMap.ini"

extract-map:
	cargo run -- extract -i "fixtures/Dispel/Ref/Map.ini"

extract-monsters:
	cargo run -- extract -i "fixtures/Dispel/MonsterInGame/Monster.db"

extract-heal-item:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/HealItem.db"

extract-draw-item:
	cargo run -- extract -i "fixtures/Dispel/Ref/DRAWITEM.ref"

extract-misc-item:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/MiscItem.db"

extract-edit-item:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/EditItem.db"

extract-event-item:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/EventItem.db"

extract-store-db:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/STORE.DB"

extract-extra-ref:
	cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Extdun01.ref"

extract-monster-ref:
	cargo run -- extract -i "fixtures/Dispel/MonsterInGame/Mondun01.ref"

extract-npc-ref:
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/Npccat1.ref"

extract-multi-magic:
	cargo run -- extract -i "fixtures/Dispel/MagicInGame/MulMagic.db"

extract-party-level:
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/PrtLevel.db"

extract-event-npc-ref:
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/Eventnpc.ref"

extract-party-ini:
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/PrtIni.db"

extract-quest:
	cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Quest.scr"

extract-message:
	cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Message.scr"

extract-ch-data:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/ChData.db"

extract-dialog:
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/Dlgcat1.dlg"

extract-dialogue-text:
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/PartyPgp.pgp"

extract-weapons:
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/weaponItem.db"

database-import:
	cargo run -- database import "fixtures/Dispel" "database.sqlite"

dialogs:
	# Show dialog flow (without text content)
	# cargo run -- dialog fixtures/Dispel/NpcInGame/Dlg$(map_id).dlg

	# Show dialog flow with text from PGP file
	# cargo run -- dialog fixtures/Dispel/NpcInGame/Dlg$(map_id).dlg -p fixtures/Dispel/NpcInGame/Pgp$(map_id).pgp

	# Show dialog flow with NPC names from NPC ref file
	# cargo run -- dialog fixtures/Dispel/NpcInGame/Dlg$(map_id).dlg -n fixtures/Dispel/NpcInGame/Npc$(map_id).ref

	# Full usage with all options
	cargo run -- dialog fixtures/Dispel/NpcInGame/Dlg$(map_id).dlg -p fixtures/Dispel/NpcInGame/Pgp$(map_id).pgp -n fixtures/Dispel/NpcInGame/Npc$(map_id).ref

extracts-help:
	cargo run -- extract --help
	cargo run -- extract -i "fixtures/Dispel/AllMap.ini"
	cargo run -- extract -i "fixtures/Dispel/Ref/Map.ini"
	cargo run -- extract -i "fixtures/Dispel/Extra.ini"
	cargo run -- extract -i "fixtures/Dispel/Event.ini"
	cargo run -- extract -i "fixtures/Dispel/Monster.ini"
	cargo run -- extract -i "fixtures/Dispel/Npc.ini"
	cargo run -- extract -i "fixtures/Dispel/Wave.ini"
	cargo run -- extract -i "fixtures/Dispel/Ref/PartyRef.ref"
	cargo run -- extract -i "fixtures/Dispel/Ref/DRAWITEM.ref"
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/Dlgcat1.dlg"
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/PartyPgp.pgp"
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/weaponItem.db"
	cargo run -- extract -i "fixtures/Dispel/MonsterInGame/Monster.db"
	# cargo run -- extract -i "fixtures/Dispel/MagicInGame/MulMagic.db"
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/STORE.DB"
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/Npccat1.ref"
	cargo run -- extract -i "fixtures/Dispel/MonsterInGame/Mondun01.ref"
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/MiscItem.db"
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/HealItem.db"
	cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Extdun01.ref"
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/EventItem.db"
	cargo run -- extract -i "fixtures/Dispel/CharacterInGame/EditItem.db"
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/PrtLevel.db"
	cargo run -- extract -i "fixtures/Dispel/NpcInGame/Eventnpc.ref"

help:
	cargo run -- --help
