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
use rusqlite::{params, Connection};
use std::collections::HashMap;
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
            DatabaseCommands::Sprites { game_path, db_path } => {
                with_connection(db_path, |conn| {
                    import_sprite_files(Path::new(game_path), conn)
                })?;
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
    eprintln!("Creating the tables...");

    let mut conn = Connection::open(db_path)?;
    initialize_database(&conn)?;

    eprintln!("Saving all data...");

    import_refs(game_path, &mut conn)?;
    // Maps must be imported before import_rest because draw_items has a FK
    // referencing maps(id) ON DELETE CASCADE.
    import_maps(game_path, &mut conn)?;
    // Databases (especially messages) must be imported before import_rest
    // because extra_refs.message_id REFERENCES messages(id) ON DELETE SET NULL.
    import_databases(game_path, &mut conn)?;
    import_dialogues_paragraphs(game_path, &mut conn)?;
    import_event_scripts(game_path, &mut conn)?;
    import_rest(game_path, &mut conn)?;
    // import_sprite_files(game_path, &mut conn)?;

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
                                if let Err(e) = dispel_core::map::save_to_db(
                                    conn,
                                    map_id,
                                    &map_data,
                                    &mut reader,
                                ) {
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
    // Insert stub event rows for self-referencing and forward-referencing
    // required_event_id values so the self-referential FK passes.
    {
        let req_ids: Vec<i32> = events
            .iter()
            .map(|e| e.required_event_id)
            .filter(|id| *id > 0)
            .collect();
        for &req_id in &req_ids {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM events WHERE event_id = ?1",
                    params![req_id],
                    |_| Ok(()),
                )
                .is_ok();
            if !exists {
                let mut stmt = conn.prepare(include_str!("../queries/insert_event.sql"))?;
                stmt.execute(params![
                    req_id,
                    0,                            // required_event_id
                    Option::<i32>::None,           // event_type_id
                    Option::<String>::None,        // event_filename
                    0,                             // counter
                ])?;
            }
        }
    }
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
    println!("Saving dialogue_script_files...");
    let mut pgp_to_file_id: HashMap<String, i32> = HashMap::new();
    {
        let mut stmt = conn.prepare(include_str!("../queries/insert_dialogue_script_file.sql"))?;
        for (file_id, dialog_file) in dialog_files.iter().enumerate() {
            let map_name = std::path::Path::new(dialog_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix("Dlg"))
                .map(|s| s.to_string());
            let pgp_path = {
                let path = std::path::Path::new(dialog_file);
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let pgp_stem = stem.replace("Dlg", "Pgp");
                let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
                format!("{}/{}.pgp", parent.display(), pgp_stem)
            };
            stmt.execute(params![file_id as i32, dialog_file, pgp_path, map_name])?;
            pgp_to_file_id.insert(pgp_path, file_id as i32);
        }
    }
    // Dialogue paragraphs must be inserted before dialogue scripts,
    // because dialogue_scripts has a FK referencing dialogue_paragraphs(file_id, id).
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
    for pgp_file in &pgp_files {
        let texts = dispel_core::references::dialogue_paragraph::read_dialogue_paragraphs(
            &main_path.join(pgp_file),
        )?;
        let file_id = pgp_to_file_id.get(*pgp_file).copied().unwrap_or_else(|| {
            panic!("No dialogue_script_files entry found for PGP file: {}", pgp_file)
        });
        save_dialogue_paragraphs(conn, file_id, &texts)?;
    }
    // Parse all dialog scripts, insert stub rows for any forward-referenced
    // IDs (paragraphs, next_dialog_id*) that don't exist yet, then save.
    println!("Saving dialogs...");
    let mut all_dialogs: Vec<(i32, Vec<dispel_core::references::dialogue_script::DialogueScript>)> =
        Vec::new();
    for (file_id, dialog_file) in dialog_files.iter().enumerate() {
        let path = main_path.join(dialog_file);
        let dialogs = dispel_core::references::dialogue_script::read_dialogs(&path)?;
        if dialogs.is_empty() {
            continue;
        }
        let file_id_i32 = file_id as i32;

        // ── Stub 1: missing dialogue_paragraphs rows ──────────────────────
        let dialog_ids: Vec<i32> = dialogs
            .iter()
            .filter_map(|d| d.dialog_id)
            .filter(|id| *id > 0)
            .collect();
        for &dialog_id in &dialog_ids {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM dialogue_paragraphs WHERE file_id = ?1 AND id = ?2",
                    params![file_id_i32, dialog_id],
                    |_| Ok(()),
                )
                .is_ok();
            if !exists {
                let mut stmt =
                    conn.prepare(include_str!("../queries/insert_dialogue_paragraphs.sql"))?;
                stmt.execute(params![
                    file_id_i32,
                    dialog_id,
                    Option::<String>::None,  // text
                    Option::<String>::None,  // comment
                    0,                       // param1
                    Option::<i32>::None,     // wave_ini_entry_id
                ])?;
            }
        }

        // ── Stub 2: forward-referenced dialogue_scripts rows ──────────────
        // next_dialog_id1/2/3 may reference scripts that haven't been
        // inserted yet (either in this file or across files).  Insert stub
        // rows so the self-referential FK passes.  INSERT OR REPLACE in
        // save_dialogs will overwrite them with real data.
        let next_ids: Vec<i32> = dialogs
            .iter()
            .flat_map(|d| [d.next_dialog_id1, d.next_dialog_id2, d.next_dialog_id3])
            .flatten()
            .filter(|id| *id > 0)
            .collect();
        for &next_id in &next_ids {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM dialogue_scripts WHERE dialog_file_id = ?1 AND id = ?2",
                    params![file_id_i32, next_id],
                    |_| Ok(()),
                )
                .is_ok();
            if !exists {
                // Only the PK columns matter — other fields are nullable.
                let mut stmt =
                    conn.prepare(include_str!("../queries/insert_dialogue_scripts.sql"))?;
                stmt.execute(params![
                    file_id_i32,
                    next_id,
                    Option::<i32>::None, // required_event_id
                    Option::<i32>::None, // next_dialog_to_check
                    Option::<i32>::None, // dialog_type_id
                    Option::<i32>::None, // dialog_owner
                    Option::<i32>::None, // dialog_id
                    Option::<i32>::None, // next_dialog_id1
                    Option::<i32>::None, // next_dialog_id2
                    Option::<i32>::None, // next_dialog_id3
                    Option::<i32>::None, // triggered_event_id
                ])?;
            }
        }

        all_dialogs.push((file_id_i32, dialogs));
    }
    // Now save all dialog scripts — stubs guarantee FK compliance.
    // INSERT OR REPLACE overwrites the stubs with real data.
    for (file_id, dialogs) in &all_dialogs {
        save_dialogs(conn, *file_id, dialogs)?;
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

    // Monsters must be saved after magic_spells because
    // known_spell_slot1/2/3 REFERENCES magic_spells(id).
    println!("Saving monsters...");
    let monsters = dispel_core::references::monster_db::read_monster_db(
        &main_path.join("MonsterInGame/Monster.db"),
    )?;
    save_monsters(conn, &monsters)?;

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
    // Insert stub NPC entries for any npc_id referenced by party_refs that
    // doesn't exist in the imported Npc.ini fixture.
    {
        let npc_ids: Vec<i32> = party_refs.iter().map(|pr| pr.npc_id).collect();
        for &npc_id in &npc_ids {
            let exists: bool = conn
                .query_row("SELECT 1 FROM npc_inis WHERE id = ?1", params![npc_id], |_| Ok(()))
                .is_ok();
            if !exists {
                let mut stmt = conn.prepare(include_str!("../queries/insert_npc_ini.sql"))?;
                stmt.execute(params![
                    npc_id,
                    Option::<String>::None,  // sprite_filename
                    Option::<String>::None,  // description
                ])?;
            }
        }
    }
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
    {
        let mut stmt = conn.prepare(include_str!("../queries/insert_npc_ref_file.sql"))?;
        for (file_id, npc_ref_file) in npc_ref_files.iter().enumerate() {
            let map_name = std::path::Path::new(npc_ref_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix("Npc").or_else(|| s.strip_prefix("npc")))
                .map(|s| s.to_string());
            stmt.execute(params![file_id as i32, npc_ref_file, map_name])?;
        }
    }
    for (file_id, npc_ref_file) in npc_ref_files.iter().enumerate() {
        let npcrefs =
            dispel_core::references::npc_ref::read_npc_ref(&main_path.join(npc_ref_file))?;
        // Resolve dialog_file_id from the map name shared between npc_ref_files
        // and dialogue_script_files (e.g. Npcmap1.ref ↔ Dlgmap1.dlg both → map1).
        let map_name = std::path::Path::new(npc_ref_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("Npc").or_else(|| s.strip_prefix("npc")))
            .map(|s| s.to_string());
        let dialog_file_id: i32 = conn
            .query_row(
                "SELECT id FROM dialogue_script_files WHERE map_name = ?1",
                params![map_name],
                |row| row.get(0),
            )
            .unwrap_or(0);
        // Insert stub dialogue_scripts rows for any dialog_id that references
        // a script not yet present in the target dialogue file.
        let dialog_ids: Vec<i32> = npcrefs
            .iter()
            .map(|n| n.dialog_id)
            .filter(|id| *id > 0)
            .collect();
        for &dialog_id in &dialog_ids {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM dialogue_scripts WHERE dialog_file_id = ?1 AND id = ?2",
                    params![dialog_file_id, dialog_id],
                    |_| Ok(()),
                )
                .is_ok();
            if !exists {
                let mut stmt =
                    conn.prepare(include_str!("../queries/insert_dialogue_scripts.sql"))?;
                stmt.execute(params![
                    dialog_file_id,
                    dialog_id,
                    Option::<i32>::None, // required_event_id
                    Option::<i32>::None, // next_dialog_to_check
                    Option::<i32>::None, // dialog_type_id
                    Option::<i32>::None, // dialog_owner
                    Option::<i32>::None, // dialog_id
                    Option::<i32>::None, // next_dialog_id1
                    Option::<i32>::None, // next_dialog_id2
                    Option::<i32>::None, // next_dialog_id3
                    Option::<i32>::None, // triggered_event_id
                ])?;
            }
        }
        save_npc_refs(conn, file_id as i32, dialog_file_id, &npcrefs)?;
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
    {
        let mut stmt = conn.prepare(include_str!("../queries/insert_monster_ref_file.sql"))?;
        for (file_id, monster_ref_file) in monster_ref_files.iter().enumerate() {
            let map_name = std::path::Path::new(monster_ref_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix("Mon").or_else(|| s.strip_prefix("mon")))
                .map(|s| s.to_string());
            stmt.execute(params![file_id as i32, monster_ref_file, map_name])?;
        }
    }
    for (file_id, monster_ref_file) in monster_ref_files.iter().enumerate() {
        let monster_refs = dispel_core::references::monster_ref::read_monster_ref(
            &main_path.join(monster_ref_file),
        )?;
        save_monster_refs(conn, file_id as i32, &monster_refs)?;
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
    {
        let mut stmt = conn.prepare(include_str!("../queries/insert_extra_ref_file.sql"))?;
        for (file_id, extra_ref_file) in extra_ref_files.iter().enumerate() {
            let map_name = std::path::Path::new(extra_ref_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix("Ext"))
                .map(|s| s.to_string());
            stmt.execute(params![file_id as i32, extra_ref_file, map_name])?;
        }
    }
    for (file_id, extra_ref_file) in extra_ref_files.iter().enumerate() {
        let extra_refs =
            dispel_core::references::extra_ref::read_extra_ref(&main_path.join(extra_ref_file))?;
        save_extra_refs(conn, file_id as i32, &extra_refs)?;
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

fn import_sprite_files(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    use image::ImageEncoder;
    use std::io::{Seek, SeekFrom};

    // Ensure the schema exists (idempotent — uses CREATE TABLE IF NOT EXISTS)
    initialize_database(conn)?;

    println!("Importing sprite files...");

    let file_insert_sql = include_str!("../queries/insert_sprite_file.sql");
    let frame_insert_sql = include_str!("../queries/insert_sprite_frame.sql");
    let seq_insert_sql = include_str!("../queries/insert_sprite_sequence.sql");
    let id_query_sql = "SELECT id FROM sprite_files WHERE normalized_path = ?1";

    let mut count = 0u64;
    let mut errors = 0u64;

    visit_dirs(main_path, &mut |entry| {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("SPR")
            && path.extension().and_then(|s| s.to_str()) != Some("spr")
        {
            return Ok(());
        }
        let normalized = path
            .strip_prefix(main_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        // 1. Upsert sprite_files entry and get its integer ID
        if conn
            .execute(file_insert_sql, rusqlite::params![&normalized])
            .is_err()
        {
            errors += 1;
            return Ok(());
        }
        let sprite_file_id: i64 =
            match conn.query_row(id_query_sql, rusqlite::params![&normalized], |row| {
                row.get(0)
            }) {
                Ok(id) => id,
                Err(_) => {
                    errors += 1;
                    return Ok(());
                }
            };

        // 2. Prepare frame + sequence statements (per-file to avoid
        //    borrow conflicts with conn inside the loop body)
        let mut frame_stmt = match conn.prepare(frame_insert_sql) {
            Ok(s) => s,
            Err(_) => {
                errors += 1;
                return Ok(());
            }
        };
        let mut seq_stmt = match conn.prepare(seq_insert_sql) {
            Ok(s) => s,
            Err(_) => {
                errors += 1;
                return Ok(());
            }
        };

        match std::fs::File::open(&path) {
            Ok(file) => {
                let file_len = match file.metadata() {
                    Ok(m) => m.len(),
                    Err(_) => {
                        errors += 1;
                        return Ok(());
                    }
                };
                let mut reader = std::io::BufReader::new(file);

                // Skip 268-byte header
                if reader.seek(SeekFrom::Start(268)).is_err() {
                    errors += 1;
                    return Ok(());
                }

                let mut seq_idx = 0;
                loop {
                    let pos = reader.stream_position().unwrap_or(file_len);
                    match dispel_core::sprite::seek_next_sequence(&mut reader, pos, file_len) {
                        Ok(true) => {}
                        _ => break,
                    }
                    let info = match dispel_core::sprite::get_sequence_info(&mut reader) {
                        Ok(i) => i,
                        Err(_) => break,
                    };

                    let frame_count = info.frame_infos.len() as i32;
                    let mut has_frame_data = false;

                    for (frame_idx, fi) in info.frame_infos.iter().enumerate() {
                        if fi.width <= 0 || fi.height <= 0 {
                            continue;
                        }
                        let img = match dispel_core::sprite::render_frame_to_rgba(
                            &mut reader,
                            fi,
                            fi.width.unsigned_abs(),
                            fi.height.unsigned_abs(),
                            0,
                            0,
                        ) {
                            Ok(img) => img,
                            Err(_) => {
                                errors += 1;
                                continue;
                            }
                        };

                        let mut png_data = Vec::new();
                        {
                            let mut cursor = std::io::Cursor::new(&mut png_data);
                            let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
                            if encoder
                                .write_image(
                                    img.as_raw(),
                                    fi.width.unsigned_abs(),
                                    fi.height.unsigned_abs(),
                                    image::ColorType::Rgba8,
                                )
                                .is_err()
                            {
                                errors += 1;
                                continue;
                            }
                        }

                        if frame_stmt
                            .execute(rusqlite::params![
                                sprite_file_id,
                                seq_idx,
                                frame_idx as i32,
                                &png_data,
                                fi.width,
                                fi.height,
                                fi.origin_x,
                                fi.origin_y,
                            ])
                            .is_err()
                        {
                            errors += 1;
                        }
                        has_frame_data = true;
                    }

                    if has_frame_data {
                        let first = &info.frame_infos[0];
                        if seq_stmt
                            .execute(rusqlite::params![
                                sprite_file_id,
                                seq_idx,
                                frame_count,
                                first.width,
                                first.height,
                                first.origin_x,
                                first.origin_y,
                            ])
                            .is_err()
                        {
                            errors += 1;
                        }
                    }

                    if reader
                        .seek(SeekFrom::Start(info.sequence_end_position))
                        .is_err()
                    {
                        break;
                    }
                    seq_idx += 1;
                }

                count += 1;
            }
            Err(e) => {
                eprintln!("WARNING: open failed for {}: {}", path.display(), e);
                errors += 1;
            }
        }
        Ok(())
    })?;

    println!("Imported {} sprite files ({} errors)", count, errors);
    Ok(())
}

/// Recursively visits all files under `dir`, calling `f` on each directory entry.
#[allow(clippy::type_complexity)]
fn visit_dirs(
    dir: &Path,
    f: &mut dyn FnMut(&std::fs::DirEntry) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, f)?;
            } else {
                f(&entry)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dispel_core::database::initialize_database;
    use std::path::Path;

    /// Creates an in-memory SQLite database, initializes the full schema, and
    /// runs every import stage (`import_refs`, `import_rest`,
    /// `import_dialogues_paragraphs`, `import_databases`,
    /// `import_event_scripts`, `import_maps`) using the game fixtures on disk.
    ///
    /// Each stage is guarded by a `.exists()` check so the test passes even
    /// when fixtures are not present (e.g. on CI) — missing stages are skipped
    /// with an `eprintln!` message.
    ///
    /// Verifies that:
    /// - No panics or SQLite errors occur during any import stage.
    /// - Foreign keys are enabled and `PRAGMA foreign_key_check` reports zero
    ///   violations after all imports.
    /// - Tables that were imported have at least one row.
    #[test]
    fn test_in_memory_database_import() {
        let game_path = Path::new("fixtures/Dispel");

        // Early exit when there are no fixtures at all — lets the test pass
        // in environments where the binary game data hasn't been downloaded.
        if !game_path.exists() {
            eprintln!(
                "Skipping test_in_memory_database_import: \
                 fixtures not found at {game_path:?}"
            );
            return;
        }

        let mut conn =
            Connection::open_in_memory().expect("Failed to create in-memory database");

        // Initialise the full schema (drops + recreates all tables).
        initialize_database(&conn)
            .expect("Failed to initialise database schema");

        // Verify foreign keys were enabled by the initialisation PRAGMAs.
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("Failed to query foreign_keys pragma");
        assert_eq!(
            fk_enabled, 1,
            "Foreign keys must be ON after initialise_database"
        );

        // ── Import reference INI files ────────────────────────────────────
        if game_path.join("Extra.ini").exists() {
            import_refs(game_path, &mut conn)
                .expect("import_refs (INI files) should succeed");
        } else {
            eprintln!("Skipping import_refs — fixtures not found");
        }

        // ── Import maps + map INIs (must be before import_rest because
        //    draw_items has a FK referencing maps(id)). ────────────────────
        if game_path.join("AllMap.ini").exists() {
            import_maps(game_path, &mut conn)
                .expect("import_maps should succeed");
        } else {
            eprintln!("Skipping import_maps — fixtures not found");
        }

        // ── Import binary databases (.db) — must be before import_rest
        //    because extra_refs.message_id REFERENCES messages(id). ────────
        if game_path.join("CharacterInGame/weaponItem.db").exists() {
            import_databases(game_path, &mut conn)
                .expect("import_databases should succeed");
        } else {
            eprintln!("Skipping import_databases — fixtures not found");
        }

        // ── Import dialogue scripts + paragraphs ──────────────────────────
        if game_path.join("NpcInGame/Dlgcat1.dlg").exists() {
            import_dialogues_paragraphs(game_path, &mut conn)
                .expect("import_dialogues_paragraphs should succeed");
        } else {
            eprintln!("Skipping import_dialogues_paragraphs — fixtures not found");
        }

        // ── Import event scripts (Ref/Event*.scr) ─────────────────────────
        if game_path.join("Ref").exists() {
            import_event_scripts(game_path, &mut conn)
                .expect("import_event_scripts should succeed");
        } else {
            eprintln!("Skipping import_event_scripts — fixtures not found");
        }

        // ── Import REF / binary placement files (must be last — depends on
        //    maps, messages, extras, events from earlier stages). ──────────
        if game_path.join("Ref/PartyRef.ref").exists() {
            import_rest(game_path, &mut conn)
                .expect("import_rest (REF files) should succeed");
        } else {
            eprintln!("Skipping import_rest — fixtures not found");
        }

        // ── Foreign-key integrity check ────────────────────────────────────
        // PRAGMA foreign_key_check returns one row per violation with columns
        // (table, rowid, parent, fkid).  No rows = clean.
        let fk_violations: Vec<String> = {
            let mut stmt = conn
                .prepare("PRAGMA foreign_key_check")
                .expect("Failed to prepare foreign_key_check");
            let rows = stmt
                .query_map([], |row| {
                    let table: String = row.get(0)?;
                    let rowid: i64 = row.get(1)?;
                    let parent: String = row.get(2)?;
                    let fkid: i64 = row.get(3)?;
                    Ok(format!("{table}.rowid={rowid} → {parent} (fkid={fkid})"))
                })
                .expect("foreign_key_check query failed");
            rows.filter_map(|r| r.ok()).collect()
        };
        assert!(
            fk_violations.is_empty(),
            "Foreign key violations detected:\n  {}",
            fk_violations.join("\n  ")
        );

        // ── Sanity checks — tables that were imported have data ────────────
        if game_path.join("Extra.ini").exists() {
            let extras_count: i32 = conn
                .query_row("SELECT COUNT(*) FROM extras", [], |row| row.get(0))
                .expect("Failed to query extras");
            assert!(extras_count > 0, "extras table should be populated");
        }

        if game_path.join("AllMap.ini").exists() {
            let map_count: i32 = conn
                .query_row("SELECT COUNT(*) FROM maps", [], |row| row.get(0))
                .expect("Failed to query maps");
            assert!(map_count > 0, "maps table should be populated");
        }

        if game_path.join("CharacterInGame/weaponItem.db").exists() {
            let weapon_count: i32 = conn
                .query_row("SELECT COUNT(*) FROM weapons", [], |row| row.get(0))
                .expect("Failed to query weapons");
            assert!(weapon_count > 0, "weapons table should be populated");
        }

        if game_path.join("NpcInGame/Dlgcat1.dlg").exists() {
            let dlg_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM dialogue_scripts",
                    [],
                    |row| row.get(0),
                )
                .expect("Failed to query dialogue_scripts");
            assert!(dlg_count > 0, "dialogue_scripts table should be populated");
        }

        if game_path.join("Ref").exists() {
            let escr_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM event_scripts",
                    [],
                    |row| row.get(0),
                )
                .expect("Failed to query event_scripts");
            assert!(escr_count > 0, "event_scripts table should be populated");
        }
    }
}
