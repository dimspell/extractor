use super::Command;
use crate::cli::DatabaseCommands;
use dispel_core::database::initialize_database;
use dispel_core::references::all_map_ini::save_maps;
use dispel_core::references::dialogue_paragraph::save_dialogue_paragraphs;
use dispel_core::references::dialogue_script::save_dialogs;
use dispel_core::references::draw_item::save_draw_items;
use dispel_core::references::edit_item_db::save_edit_items;
use dispel_core::references::event_ini::save_events;
use dispel_core::references::event_item_db::save_event_items;
use dispel_core::references::event_npc_ref::save_event_npc_refs;
use dispel_core::references::extra_ini::save_extras;
use dispel_core::references::extra_ref::save_extra_refs;
use dispel_core::references::heal_item_db::save_heal_items;
use dispel_core::references::magic_db::save_magic_spells;
use dispel_core::references::map_ini::save_map_inis;
use dispel_core::references::message_scr::save_messages;
use dispel_core::references::misc_item_db::save_misc_items;
use dispel_core::references::monster_db::save_monsters;
use dispel_core::references::monster_ini::save_monster_inis;
use dispel_core::references::monster_ref::save_monster_refs;
use dispel_core::references::npc_ini::save_npc_inis;
use dispel_core::references::npc_ref::save_npc_refs;
use dispel_core::references::party_ini_db::save_party_inis;
use dispel_core::references::party_level_db::save_party_levels;
use dispel_core::references::party_ref::save_party_refs;
use dispel_core::references::quest_scr::save_quests;
use dispel_core::references::store_db::save_stores;
use dispel_core::references::wave_ini::save_wave_inis;
use dispel_core::references::weapons_db::save_weapons;
use rusqlite::Connection;
use std::error::Error;
use std::path::Path;

/// Database command implementation
pub struct DatabaseCommand {
    pub subcommand: DatabaseCommands,
}

impl Command for DatabaseCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.subcommand {
            DatabaseCommands::Import { game_path, db_path } => {
                save_all(Path::new(game_path), db_path)?;
            }
            DatabaseCommands::DialogTexts { game_path, db_path } => {
                with_connection(db_path, |conn| {
                    import_dialogues_paragraphs(Path::new(game_path), conn)
                })?;
            }
            DatabaseCommands::Maps { game_path, db_path } => {
                with_connection(db_path, |conn| import_maps(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Databases { game_path, db_path } => {
                with_connection(db_path, |conn| import_databases(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Refs { game_path, db_path } => {
                with_connection(db_path, |conn| import_refs(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Rest { game_path, db_path } => {
                with_connection(db_path, |conn| import_rest(Path::new(game_path), conn))?;
            }
        }
        Ok(())
    }
}

fn with_connection(
    db_path: &str,
    f: impl FnOnce(&mut Connection) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let mut conn = Connection::open(db_path)?;
    f(&mut conn)?;
    let _ = conn.close();
    Ok(())
}

fn save_all(game_path: &Path, db_path: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("Saving all data...");

    let mut conn = Connection::open(db_path)?;
    initialize_database(&conn)?;

    import_refs(game_path, &mut conn)?;
    import_rest(game_path, &mut conn)?;
    import_dialogues_paragraphs(game_path, &mut conn)?;
    import_databases(game_path, &mut conn)?;
    import_event_scripts(game_path, &mut conn)?;
    import_maps(game_path, &mut conn)?;

    let _ = conn.close();
    Ok(())
}

fn import_maps(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving maps...");
    let maps =
        dispel_core::references::all_map_ini::read_all_map_ini(&main_path.join("AllMap.ini"))?;
    save_maps(conn, &maps)?;

    println!("Importing all .map files...");
    let map_dir = main_path.join("Map");
    if map_dir.exists() {
        let map_files = [
            "cat1.map",
            "cat2.map",
            "cat3.map",
            "catp.map",
            "dun01.map",
            "dun02.map",
            "dun03.map",
            "dun04.map",
            "dun05.map",
            "dun06.map",
            "dun07.map",
            "dun08.map",
            "dun09.map",
            "dun10.map",
            "dun11.map",
            "dun12.map",
            "dun13.map",
            "dun14.map",
            "dun15.map",
            "dun16.map",
            "dun17.map",
            "dun18.map",
            "dun19.map",
            "dun20.map",
            "dun21.map",
            "dun22.map",
            "dun23.map",
            "dun24.map",
            "dun25.map",
            "final.map",
            "map1.map",
            "map2.map",
            "map3.map",
        ];
        for entry in map_files {
            let path = map_dir.join(entry);
            if path.extension().and_then(|s| s.to_str()) == Some("map") {
                let map_id = path.file_stem().unwrap().to_str().unwrap();
                if map_id == "map4" {
                    continue;
                }
                println!("Importing map file: {}", path.display());
                match std::fs::File::open(&path) {
                    Ok(file) => {
                        let mut reader = std::io::BufReader::new(file);
                        match dispel_core::map::read_map_data(&mut reader) {
                            Ok(map_data) => {
                                if let Err(e) =
                                    dispel_core::map::save_to_db(conn, map_id, &map_data)
                                {
                                    eprintln!(
                                        "WARNING: could not save map {} to database: {}",
                                        map_id, e
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "WARNING: could not read map data from {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WARNING: could not open map file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    println!("Saving map_inis...");
    let map_inis = dispel_core::references::map_ini::read_map_ini(&main_path.join("Ref/Map.ini"))?;
    save_map_inis(conn, &map_inis)?;
    Ok(())
}

fn import_refs(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving extras...");
    let extras = dispel_core::references::extra_ini::read_extra_ini(&main_path.join("Extra.ini"))?;
    save_extras(conn, &extras)?;
    println!("Saving events...");
    let events = dispel_core::references::event_ini::read_event_ini(&main_path.join("Event.ini"))?;
    save_events(conn, &events)?;
    println!("Saving monster_inis...");
    let monster_inis =
        dispel_core::references::monster_ini::read_monster_ini(&main_path.join("Monster.ini"))?;
    save_monster_inis(conn, &monster_inis)?;
    println!("Saving npc_inis...");
    let npc_inis = dispel_core::references::npc_ini::read_npc_ini(&main_path.join("Npc.ini"))?;
    save_npc_inis(conn, &npc_inis)?;
    println!("Saving wave_inis...");
    let wave_inis = dispel_core::references::wave_ini::read_wave_ini(&main_path.join("Wave.ini"))?;
    save_wave_inis(conn, &wave_inis)?;
    Ok(())
}

fn import_dialogues_paragraphs(
    main_path: &Path,
    conn: &mut Connection,
) -> Result<(), Box<dyn Error>> {
    let dialog_files = [
        "NpcInGame/Dlgcat1.dlg",
        "NpcInGame/Dlgcat2.dlg",
        "NpcInGame/Dlgcat3.dlg",
        "NpcInGame/Dlgcatp.dlg",
        "NpcInGame/Dlgdun04.dlg",
        "NpcInGame/Dlgdun07.dlg",
        "NpcInGame/Dlgdun08.dlg",
        "NpcInGame/Dlgdun10.dlg",
        "NpcInGame/Dlgdun19.dlg",
        "NpcInGame/Dlgdun22.dlg",
        "NpcInGame/Dlgmap1.dlg",
        "NpcInGame/Dlgmap2.dlg",
        "NpcInGame/Dlgmap3.dlg",
        "NpcInGame/PartyDlg.dlg",
    ];
    println!("Saving dialogs...");
    for dialog_file in dialog_files {
        let dialogs =
            dispel_core::references::dialogue_script::read_dialogs(&main_path.join(dialog_file))?;
        save_dialogs(conn, dialog_file, &dialogs)?;
    }

    let pgp_files = [
        "NpcInGame/PartyPgp.pgp",
        "NpcInGame/Pgpcat1.pgp",
        "NpcInGame/Pgpcat2.pgp",
        "NpcInGame/Pgpcat3.pgp",
        "NpcInGame/Pgpcatp.pgp",
        "NpcInGame/Pgpdun04.pgp",
        "NpcInGame/Pgpdun07.pgp",
        "NpcInGame/Pgpdun08.pgp",
        "NpcInGame/Pgpdun10.pgp",
        "NpcInGame/Pgpdun19.pgp",
        "NpcInGame/Pgpdun22.pgp",
        "NpcInGame/Pgpmap1.pgp",
        "NpcInGame/Pgpmap2.pgp",
        "NpcInGame/Pgpmap3.pgp",
        "NpcInGame/PartyPgp.pgp",
    ];
    println!("Saving dialogue texts...");
    for pgp_file in pgp_files {
        let texts = dispel_core::references::dialogue_paragraph::read_dialogue_paragraphs(
            &main_path.join(pgp_file),
        )?;
        save_dialogue_paragraphs(conn, pgp_file, &texts)?;
    }
    Ok(())
}

fn import_databases(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving weapons...");
    let weapons = dispel_core::references::weapons_db::read_weapons_db(
        &main_path.join("CharacterInGame/weaponItem.db"),
    )?;
    save_weapons(conn, &weapons)?;
    println!("Saving stores...");
    let stores = dispel_core::references::store_db::read_store_db(
        &main_path.join("CharacterInGame/STORE.DB"),
    )?;
    save_stores(conn, &stores)?;
    println!("Saving monsters...");
    let monsters = dispel_core::references::monster_db::read_monster_db(
        &main_path.join("MonsterInGame/Monster.db"),
    )?;
    save_monsters(conn, &monsters)?;
    println!("Saving misc_items...");
    let misc_items = dispel_core::references::misc_item_db::read_misc_item_db(
        &main_path.join("CharacterInGame/MiscItem.db"),
    )?;
    save_misc_items(conn, &misc_items)?;
    println!("Saving heal_items...");
    let heal_items = dispel_core::references::heal_item_db::read_heal_item_db(
        &main_path.join("CharacterInGame/HealItem.db"),
    )?;
    save_heal_items(conn, &heal_items)?;
    println!("Saving event_items...");
    let event_items = dispel_core::references::event_item_db::read_event_item_db(
        &main_path.join("CharacterInGame/EventItem.db"),
    )?;
    save_event_items(conn, &event_items)?;

    println!("Saving edit_items...");
    let edit_items = dispel_core::references::edit_item_db::read_edit_item_db(
        &main_path.join("CharacterInGame/EditItem.db"),
    )?;
    save_edit_items(conn, &edit_items)?;

    println!("Saving party_level_db...");
    let party_levels = dispel_core::references::party_level_db::read_party_level_db(
        &main_path.join("NpcInGame/PrtLevel.db"),
    )?;
    save_party_levels(conn, &party_levels)?;

    println!("Saving party_ini_db...");
    let party_inis = dispel_core::references::party_ini_db::read_party_ini_db(
        &main_path.join("NpcInGame/PrtIni.db"),
    )?;
    save_party_inis(conn, &party_inis)?;

    println!("Saving magic_spells...");
    let magic_spells =
        dispel_core::references::magic_db::read_magic_db(&main_path.join("MagicInGame/Magic.db"))?;
    save_magic_spells(conn, &magic_spells)?;

    println!("Saving quests...");
    let quests =
        dispel_core::references::quest_scr::read_quests(&main_path.join("ExtraInGame/Quest.scr"))?;
    save_quests(conn, &quests)?;

    println!("Saving messages...");
    let messages = dispel_core::references::message_scr::read_messages(
        &main_path.join("ExtraInGame/Message.scr"),
    )?;
    save_messages(conn, &messages)?;

    Ok(())
}

fn import_rest(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving party_refs...");
    let party_refs =
        dispel_core::references::party_ref::read_part_refs(&main_path.join("Ref/PartyRef.ref"))?;
    save_party_refs(conn, &party_refs)?;
    println!("Saving draw_items...");
    let draw_items =
        dispel_core::references::draw_item::read_draw_items(&main_path.join("Ref/DRAWITEM.ref"))?;
    save_draw_items(conn, &draw_items)?;

    let npc_ref_files = [
        "NpcInGame/Npccat1.ref",
        "NpcInGame/Npccat2.ref",
        "NpcInGame/Npccat3.ref",
        "NpcInGame/Npccatp.ref",
        "NpcInGame/npcdun08.ref",
        "NpcInGame/npcdun19.ref",
        "NpcInGame/Npcmap1.ref",
        "NpcInGame/Npcmap2.ref",
        "NpcInGame/Npcmap3.ref",
    ];
    println!("Saving npcrefs...");
    for npc_ref_file in npc_ref_files {
        let npcrefs =
            dispel_core::references::npc_ref::read_npc_ref(&main_path.join(npc_ref_file))?;
        save_npc_refs(conn, npc_ref_file, &npcrefs)?;
    }

    println!("Saving event_npc_refs...");
    let event_npc_refs = dispel_core::references::event_npc_ref::read_event_npc_ref(
        &main_path.join("NpcInGame/Eventnpc.ref"),
    )?;
    save_event_npc_refs(conn, &event_npc_refs)?;

    let monster_ref_files = [
        "MonsterInGame/Mondun01.ref",
        "MonsterInGame/Mondun02.ref",
        "MonsterInGame/mondun03.ref",
        "MonsterInGame/mondun04.ref",
        "MonsterInGame/Mondun05.ref",
        "MonsterInGame/mondun06.ref",
        "MonsterInGame/mondun07.ref",
        "MonsterInGame/mondun08.ref",
        "MonsterInGame/mondun09.ref",
        "MonsterInGame/Mondun10.ref",
        "MonsterInGame/mondun11.ref",
        "MonsterInGame/mondun12.ref",
        "MonsterInGame/mondun13.ref",
        "MonsterInGame/Mondun14.ref",
        "MonsterInGame/mondun15.ref",
        "MonsterInGame/mondun16.ref",
        "MonsterInGame/mondun17.ref",
        "MonsterInGame/mondun18.ref",
        "MonsterInGame/Mondun19.ref",
        "MonsterInGame/mondun20.ref",
        "MonsterInGame/mondun21.ref",
        "MonsterInGame/mondun22.ref",
        "MonsterInGame/mondun23.ref",
        "MonsterInGame/mondun24.ref",
        "MonsterInGame/mondun25.ref",
        "MonsterInGame/monfinal.ref",
        "MonsterInGame/Monmap1.ref",
        "MonsterInGame/Monmap2.ref",
        "MonsterInGame/Monmap3.ref",
    ];
    println!("Saving monster_refs...");
    for monster_ref_file in monster_ref_files {
        let monster_refs = dispel_core::references::monster_ref::read_monster_ref(
            &main_path.join(monster_ref_file),
        )?;
        save_monster_refs(conn, monster_ref_file, &monster_refs)?;
    }

    let extra_ref_files = [
        "ExtraInGame/Extcat3.ref",
        "ExtraInGame/Extdun01.ref",
        "ExtraInGame/Extdun02.ref",
        "ExtraInGame/Extdun03.ref",
        "ExtraInGame/Extdun04.ref",
        "ExtraInGame/Extdun05.ref",
        "ExtraInGame/Extdun06.ref",
        "ExtraInGame/Extdun07.ref",
        "ExtraInGame/Extdun08.ref",
        "ExtraInGame/Extdun09.ref",
        "ExtraInGame/Extdun10.ref",
        "ExtraInGame/Extdun11.ref",
        "ExtraInGame/Extdun12.ref",
        "ExtraInGame/Extdun13.ref",
        "ExtraInGame/Extdun14.ref",
        "ExtraInGame/Extdun15.ref",
        "ExtraInGame/Extdun16.ref",
        "ExtraInGame/Extdun17.ref",
        "ExtraInGame/Extdun18.ref",
        "ExtraInGame/Extdun19.ref",
        "ExtraInGame/Extdun20.ref",
        "ExtraInGame/Extdun21.ref",
        "ExtraInGame/Extdun22.ref",
        "ExtraInGame/Extdun23.ref",
        "ExtraInGame/Extdun24.ref",
        "ExtraInGame/Extdun25.ref",
        "ExtraInGame/Extfinal.ref",
        "ExtraInGame/Extmap1.ref",
        "ExtraInGame/Extmap2.ref",
        "ExtraInGame/Extmap3.ref",
    ];
    println!("Saving extra_refs...");
    for extra_ref_file in extra_ref_files {
        let extra_refs =
            dispel_core::references::extra_ref::read_extra_ref(&main_path.join(extra_ref_file))?;
        save_extra_refs(conn, extra_ref_file, &extra_refs)?;
    }
    Ok(())
}

fn import_event_scripts(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving event_scripts...");
    // Find all event script files in Ref directory
    let ref_dir = main_path.join("Ref");
    if ref_dir.exists() {
        let mut event_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&ref_dir) {
            for entry in entries.flatten() {
                if let Some(path) = entry.path().to_str() {
                    if (path.contains("Event") || path.contains("event"))
                        && (path.ends_with(".scr") || path.ends_with(".SCR"))
                    {
                        event_files.push(entry.path());
                    }
                }
            }
        }

        for event_file in event_files {
            let scripts = dispel_core::references::event_scr::read_event_scripts(&event_file)?;
            dispel_core::save_event_scripts(conn, &scripts)?;
        }
    }
    Ok(())
}
