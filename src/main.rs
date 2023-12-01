use crate::database::{
    save_draw_items, save_edit_items, save_event_items, save_events, save_extra_refs, save_extras,
    save_heal_items, save_map_inis, save_maps, save_misc_items, save_monster_inis,
    save_monster_refs, save_monsters, save_npc_inis, save_npc_refs, save_party_refs, save_stores,
    save_wave_inis, save_weapons,
};
use database::{save_dialogs, save_party_pgps};
use rusqlite::Connection;
use std::path::Path;
use std::io::{prelude::*};
use std::{
    io::{self},
};

pub mod database;
pub mod map;
pub mod references;
pub mod snf;
pub mod sprite;
pub mod tileset;


fn main() -> io::Result<()> {
    // _ = snf::extract(
    //     &Path::new("sample-data/Wave/Piach4.snf"),
    //     &Path::new("Piach.wav"),
    // );

    // sprite::extract(&Path::new("sample-data/Inter/Multi/mg205.spr"), "mg205".to_string())?;

    // sprite::extract(&Path::new("sample-data/MagicInGame/Casting1.spr"))?;
    // sprite::extract(&Path::new("sample-data/CharacterInGame/M_BODY1.SPR"))?;
    // map::extract(false)?;

    // sprite::animation(&Path::new("sample-data/MagicInGame/Casting1.spr"))?;
    // sprite::animation(&Path::new("sample-data/CharacterInGame/M_BODY1.spr"))?;

    // let tiles = tileset::extract(&Path::new("sample-data/Map/cat1.gtl"))?;
    // tileset::plot_tileset_map(&tiles, "cat1-gtl.png");

    // let tiles = tileset::extract(&Path::new("sample-data/Map/cat1.btl"))?;
    // tileset::plot_tileset_map(&tiles, "cat1-btl.png");

    // let tiles = tileset::extract(&Path::new("sample-data/Map/cat1.gtl"))?;
    // tileset::plot_all_tiles(&tiles);

    // let maps = references::read_all_map_ini(&Path::new("sample-data/AllMap.ini"))?;
    // let map_inis = references::read_map_ini(&Path::new("sample-data/Ref/Map.ini"))?;
    // let extras = references::read_extra_ini(&Path::new("sample-data/Extra.ini"))?;
    // let events = references::read_event_ini(&Path::new("sample-data/Event.ini"))?;
    // let monster_inis = references::read_monster_ini(&Path::new("sample-data/Monster.ini"))?;
    // let npc_inis = references::read_npc_ini(&Path::new("sample-data/Npc.ini"))?;
    // let wave_inis = references::read_wave_ini(&Path::new("sample-data/Wave.ini"))?;
    // let party_refs = references::read_part_refs(&Path::new("sample-data/Ref/PartyRef.ref"))?;
    // let draw_items = references::read_draw_items(&Path::new("sample-data/Ref/DRAWITEM.ref"))?;
    // todo let event_npc_refs = references::read_event_npc_ref(&Path::new("sample-data/NpcInGame/Eventnpc.ref"))?;
    // let party_pgps = references::read_party_pgps(&Path::new("sample-data/NpcInGame/PartyPgp.pgp"))?;
    // let dialogs = references::read_dialogs(&Path::new("sample-data/NpcInGame/Dlgcat1.dlg"))?;
    // let dialogs = references::read_dialogs(&Path::new("sample-data/NpcInGame/PartyDlg.dlg"))?;

    // let weapons =
    //     references::read_weapons_db(&Path::new("sample-data/CharacterInGame/weaponItem.db"))?;
// for w in weapons.iter() {
    //     println!("{:?} {:?}",w.id,  w.name);
    // }

    // references::read_mutli_magic_db(&Path::new("sample-data/MagicInGame/MulMagic.db"))?;

    // let stores = references::read_store_db(&Path::new("sample-data/CharacterInGame/STORE.DB"))?;
    // let npcrefs = references::read_npc_ref(&Path::new("sample-data/NpcInGame/Npccat1.ref"))?;
    // let npcrefs = references::read_npc_ref(&Path::new("sample-data/NpcInGame/Npcmap1.ref"))?;
    // let monsters = references::read_monster_db(&Path::new("sample-data/MonsterInGame/Monster.db"))?;
    // let monster_refs =
    //     references::read_monster_ref(&Path::new("sample-data/MonsterInGame/Mondun01.ref"))?;
    // let misc_items =
    //     references::read_misc_item_db(&Path::new("sample-data/CharacterInGame/MiscItem.db"))?;
    // let heal_items =
    //     references::read_heal_item_db(&Path::new("sample-data/CharacterInGame/HealItem.db"))?;
    // let extra_refs =
    //     references::read_extra_ref(&Path::new("sample-data/ExtraInGame/Extdun01.ref"))?;
    // let event_items =
    //     references::read_event_item_db(&Path::new("sample-data/CharacterInGame/EventItem.db"))?;
    // let edit_items =
    //     references::read_edit_item_db(&Path::new("sample-data/CharacterInGame/EditItem.db"))?;
    let party_level =
        references::read_party_level_db(&Path::new("sample-data/NpcInGame/PrtLevel.db"))?;

    // database::create_database();

    // let conn = Connection::open("data.sqlite").unwrap();

    // conn.close().unwrap();

    Ok(())
}

fn save_all() -> io::Result<()> {
    let maps = references::read_all_map_ini(&Path::new("sample-data/AllMap.ini"))?;
    let map_inis = references::read_map_ini(&Path::new("sample-data/Ref/Map.ini"))?;
    let extras = references::read_extra_ini(&Path::new("sample-data/Extra.ini"))?;
    let events = references::read_event_ini(&Path::new("sample-data/Event.ini"))?;
    let monster_inis = references::read_monster_ini(&Path::new("sample-data/Monster.ini"))?;
    let npc_inis = references::read_npc_ini(&Path::new("sample-data/Npc.ini"))?;
    let wave_inis = references::read_wave_ini(&Path::new("sample-data/Wave.ini"))?;
    let party_refs = references::read_part_refs(&Path::new("sample-data/Ref/PartyRef.ref"))?;
    let draw_items = references::read_draw_items(&Path::new("sample-data/Ref/DRAWITEM.ref"))?;
    let party_pgps = references::read_party_pgps(&Path::new("sample-data/NpcInGame/PartyPgp.pgp"))?;
    let dialogs = references::read_dialogs(&Path::new("sample-data/NpcInGame/Dlgcat1.dlg"))?;

    let weapons =
        references::read_weapons_db(&Path::new("sample-data/CharacterInGame/weaponItem.db"))?;
    let stores = references::read_store_db(&Path::new("sample-data/CharacterInGame/STORE.DB"))?;
    let npcrefs = references::read_npc_ref(&Path::new("sample-data/NpcInGame/Npccat1.ref"))?;
    let monsters = references::read_monster_db(&Path::new("sample-data/MonsterInGame/Monster.db"))?;
    let monster_refs =
        references::read_monster_ref(&Path::new("sample-data/MonsterInGame/Mondun01.ref"))?;
    let misc_items =
        references::read_misc_item_db(&Path::new("sample-data/CharacterInGame/MiscItem.db"))?;
    let heal_items =
        references::read_heal_item_db(&Path::new("sample-data/CharacterInGame/HealItem.db"))?;
    let extra_refs =
        references::read_extra_ref(&Path::new("sample-data/ExtraInGame/Extdun01.ref"))?;
    let event_items =
        references::read_event_item_db(&Path::new("sample-data/CharacterInGame/EventItem.db"))?;
    let edit_items =
        references::read_edit_item_db(&Path::new("sample-data/CharacterInGame/EditItem.db"))?;

    let conn = Connection::open("database.sqlite").unwrap();

    save_maps(&conn, &maps).unwrap();
    save_events(&conn, &events).unwrap();
    save_extras(&conn, &extras).unwrap();
    save_monster_inis(&conn, &monster_inis).unwrap();
    save_npc_inis(&conn, &npc_inis).unwrap();
    save_wave_inis(&conn, &wave_inis).unwrap();
    save_map_inis(&conn, &map_inis).unwrap();
    save_party_refs(&conn, &party_refs).unwrap();
    save_draw_items(&conn, &draw_items).unwrap();
    save_party_pgps(&conn, &party_pgps).unwrap();
    save_dialogs(&conn, &dialogs).unwrap();

    save_monsters(&conn, &monsters).unwrap();
    save_stores(&conn, &stores).unwrap();
    save_weapons(&conn, &weapons).unwrap();
    save_npc_refs(&conn, &npcrefs).unwrap();
    save_monster_refs(&conn, &monster_refs).unwrap();
    save_misc_items(&conn, &misc_items).unwrap();
    save_heal_items(&conn, &heal_items).unwrap();
    save_extra_refs(&conn, &extra_refs).unwrap();
    save_event_items(&conn, &event_items).unwrap();
    save_edit_items(&conn, &edit_items).unwrap();

    conn.close().unwrap();

    Ok(())
}
