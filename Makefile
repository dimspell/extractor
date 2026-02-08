map_id ?= cat1

sound:
	cargo run -- sound --input notexist.snf --output Piach.wav

sprite-sprite:
	cargo run -- sprite \
		--mode=sprite \
		--input="fixtures/Dispel/CharacterInGame/M_BODY1.SPR" --output=""

sprite-animation:
	cargo run -- sprite \
		--mode=animation \
		--input="fixtures/Dispel/CharacterInGame/M_BODY1.SPR" \
		--output=""

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

map-from-db:
	cargo run -- map from-db --map-id ${map_id} --gtl-atlas ${map_id}.gtl.png --btl-atlas ${map_id}.btl.png -o out_${map_id}.png

ref-all-maps:
	cargo run -- ref all-maps "fixtures/Dispel/AllMap.ini"

ref-map:
	cargo run -- ref map &Path::new("sample-data/Ref/Map.ini")

ref-monsters:
	cargo run -- ref monsters "fixtures/Dispel/MonsterInGame/Monster.db"

ref-heal-item:
	cargo run -- ref heal-items "fixtures/Dispel/CharacterInGame/HealItem.db"

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

ref-party-level:
	cargo run -- ref event-npc-ref "fixtures/Dispel/NpcInGame/Eventnpc.ref"

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
	cargo run -- ref party-pgp "fixtures/Dispel/NpcInGame/PartyPgp.pgp"
	cargo run -- ref party-dialog "fixtures/Dispel/NpcInGame/PartyDlg.dlg"
	cargo run -- ref dialog "fixtures/Dispel/NpcInGame/Dlgcat1.dlg"
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
