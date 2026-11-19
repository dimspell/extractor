//! Game-file import orchestration.
//!
//! Every stage reads game files through the parsers in [`crate::references`]
//! and [`crate::map`], then writes rows using the statements registered in the
//! schema manifest. [`import_all`] runs the stages in the order required by the
//! data dependencies (e.g. dialogue file metadata must exist before NPC refs
//! can resolve their `dialog_file_id`).
//!
//! The schema declares no SQL foreign keys, so the "stub row" logic that used
//! to pad forward references has been removed; ordering is the only constraint
//! that actually matters.

use super::insert_sql;
use crate::references::all_map_ini::save_maps;
use crate::references::chdata_db::save_chdatas;
use crate::references::dialogue_paragraph::save_dialogue_paragraphs;
use crate::references::dialogue_script::save_dialogs;
use crate::references::draw_item::save_draw_items;
use crate::references::edit_item_db::save_edit_items;
use crate::references::event_ini::save_events;
use crate::references::event_item_db::save_event_items;
use crate::references::event_npc_ref::save_event_npc_refs;
use crate::references::extra_ini::save_extras;
use crate::references::extra_ref::save_extra_refs;
use crate::references::heal_item_db::save_heal_items;
use crate::references::magic_db::save_magic_spells;
use crate::references::map_ini::save_map_inis;
use crate::references::message_scr::save_messages;
use crate::references::misc_item_db::save_misc_items;
use crate::references::monster_db::save_monsters;
use crate::references::monster_ini::save_monster_inis;
use crate::references::monster_ref::save_monster_refs;
use crate::references::npc_ini::save_npc_inis;
use crate::references::npc_ref::save_npc_refs;
use crate::references::party_ini_db::save_party_inis;
use crate::references::party_level_db::save_party_levels;
use crate::references::party_ref::save_party_refs;
use crate::references::quest_scr::save_quests;
use crate::references::store_db::save_stores;
use crate::references::wave_ini::save_wave_inis;
use crate::references::weapons_db::save_weapons;
use crate::save_event_scripts;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

/// Imports every game data format into an already bootstrapped database.
///
/// Stages run in dependency order: reference INIs, maps, binary databases,
/// dialogue text + scripts, event scripts, then placement REF files. Sprite
/// import is intentionally excluded (it is a separate, opt-in CLI stage).
pub fn import_all(game_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    import_refs(game_path, conn)?;
    // Maps precede REF files: draw_items are keyed by map name.
    import_maps(game_path, conn)?;
    // Databases precede REF files: extra_refs resolve message IDs.
    import_databases(game_path, conn)?;
    import_dialogues_paragraphs(game_path, conn)?;
    import_event_scripts(game_path, conn)?;
    import_rest(game_path, conn)?;
    Ok(())
}

/// Historical map list, used only when `AllMap.ini` cannot be read or parsed.
const FALLBACK_MAP_STEMS: &[&str] = &[
    "cat1", "cat2", "cat3", "catp", "dun01", "dun02", "dun03", "dun04", "dun05", "dun06", "dun07",
    "dun08", "dun09", "dun10", "dun11", "dun12", "dun13", "dun14", "dun15", "dun16", "dun17",
    "dun18", "dun19", "dun20", "dun21", "dun22", "dun23", "dun24", "dun25", "final", "map1",
    "map2", "map3",
];

/// Everything map import needs from `AllMap.ini`: the parsed records for the
/// `maps` table and the stems of the `.map` files listed in the INI, in file
/// order. The INI is the single source of truth — a static list is only a
/// fallback for a genuinely unreadable/unparseable INI.
struct MapInventory {
    records: Vec<crate::references::all_map_ini::Map>,
    stems: Vec<String>,
}

/// AllMap.ini normally stores bare stems (`cat1`), but tolerate an extension.
fn normalize_map_stem(raw: &str) -> String {
    let raw = raw.trim();
    let stem = raw
        .strip_suffix(".map")
        .or_else(|| raw.strip_suffix(".MAP"))
        .unwrap_or(raw);
    stem.to_owned()
}

fn fallback_inventory() -> MapInventory {
    MapInventory {
        records: Vec::new(),
        stems: FALLBACK_MAP_STEMS.iter().map(|s| (*s).to_owned()).collect(),
    }
}

fn read_map_inventory(main_path: &Path) -> MapInventory {
    let ini_path = main_path.join("AllMap.ini");
    match crate::references::all_map_ini::read_all_map_ini(&ini_path) {
        Ok(records) if !records.is_empty() => MapInventory {
            stems: records
                .iter()
                .map(|m| normalize_map_stem(&m.map_filename))
                .collect(),
            records,
        },
        Ok(records) => {
            eprintln!(
                "WARNING: AllMap.ini at {} lists no maps; using fallback map list",
                ini_path.display()
            );
            MapInventory {
                records,
                ..fallback_inventory()
            }
        }
        Err(e) => {
            // Report the parse/read failure rather than silently ignoring it.
            eprintln!(
                "WARNING: could not parse AllMap.ini at {}: {e}; using fallback map list",
                ini_path.display()
            );
            fallback_inventory()
        }
    }
}

pub fn import_maps(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving maps...");
    let inventory = read_map_inventory(main_path);
    save_maps(conn, &inventory.records)?;

    println!("Importing all .map files...");
    let map_dir = main_path.join("Map");
    if map_dir.exists() {
        for stem in &inventory.stems {
            // `map4` is listed in AllMap.ini but has no geometry file in the
            // shipped game; skip it rather than failing the whole import.
            if stem == "map4" {
                continue;
            }
            let map_id = Path::new(stem)
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("invalid map file name: {stem}"))?;
            if map_id != stem {
                eprintln!("WARNING: skipping map with path in filename: {stem}");
                continue;
            }
            let path = map_dir.join(format!("{map_id}.map"));
            println!("Importing map file: {}", path.display());
            let file = std::fs::File::open(&path)?;
            let mut reader = std::io::BufReader::new(file);
            let map_data = crate::map::read_map_data(&mut reader)?;
            crate::map::save_to_db(conn, map_id, &map_data, &mut reader)?;
        }
    }
    println!("Saving fog_factors...");
    let fog_path = main_path.join("ExtraInGame/fogdata.dat");
    if !fog_path.exists() {
        eprintln!(
            "WARNING: fogdata.dat not found at {} — skipping fog_factors",
            fog_path.display()
        );
    } else {
        let fog = crate::map::FogData::load(main_path)?;
        crate::map::save_fog_to_db(conn, &fog)?;
    }

    println!("Saving map_inis...");
    let map_inis = crate::references::map_ini::read_map_ini(&main_path.join("Ref/Map.ini"))?;
    save_map_inis(conn, &map_inis)?;
    Ok(())
}

pub fn import_refs(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving extras...");
    let extras = crate::references::extra_ini::read_extra_ini(&main_path.join("Extra.ini"))?;
    save_extras(conn, &extras)?;
    println!("Saving events...");
    let events = crate::references::event_ini::read_event_ini(&main_path.join("Event.ini"))?;
    save_events(conn, &events)?;
    println!("Saving monster_inis...");
    let monster_inis =
        crate::references::monster_ini::read_monster_ini(&main_path.join("Monster.ini"))?;
    save_monster_inis(conn, &monster_inis)?;
    println!("Saving npc_inis...");
    let npc_inis = crate::references::npc_ini::read_npc_ini(&main_path.join("Npc.ini"))?;
    save_npc_inis(conn, &npc_inis)?;
    println!("Saving wave_inis...");
    let wave_inis = crate::references::wave_ini::read_wave_ini(&main_path.join("Wave.ini"))?;
    save_wave_inis(conn, &wave_inis)?;
    Ok(())
}

pub fn import_dialogues_paragraphs(
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
        let mut stmt = conn.prepare(insert_sql("dialogue_script_files"))?;
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
    // Dialogue paragraphs are written before scripts so paragraph lookups by
    // (file_id, id) can resolve them; the schema has no SQL foreign keys.
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
        let texts = crate::references::dialogue_paragraph::read_dialogue_paragraphs(
            &main_path.join(pgp_file),
        )?;
        let file_id = pgp_to_file_id.get(*pgp_file).copied().unwrap_or_else(|| {
            panic!(
                "No dialogue_script_files entry found for PGP file: {}",
                pgp_file
            )
        });
        save_dialogue_paragraphs(conn, file_id, &texts)?;
    }
    println!("Saving dialogs...");
    for (file_id, dialog_file) in dialog_files.iter().enumerate() {
        let path = main_path.join(dialog_file);
        let dialogs = crate::references::dialogue_script::read_dialogs(&path)?;
        if dialogs.is_empty() {
            continue;
        }
        save_dialogs(conn, file_id as i32, &dialogs)?;
    }
    Ok(())
}

pub fn import_databases(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving weapons...");
    let weapons = crate::references::weapons_db::read_weapons_db(
        &main_path.join("CharacterInGame/weaponItem.db"),
    )?;
    save_weapons(conn, &weapons)?;
    println!("Saving stores...");
    let stores =
        crate::references::store_db::read_store_db(&main_path.join("CharacterInGame/STORE.DB"))?;
    save_stores(conn, &stores)?;
    println!("Saving misc_items...");
    let misc_items = crate::references::misc_item_db::read_misc_item_db(
        &main_path.join("CharacterInGame/MiscItem.db"),
    )?;
    save_misc_items(conn, &misc_items)?;
    println!("Saving heal_items...");
    let heal_items = crate::references::heal_item_db::read_heal_item_db(
        &main_path.join("CharacterInGame/HealItem.db"),
    )?;
    save_heal_items(conn, &heal_items)?;
    println!("Saving event_items...");
    let event_items = crate::references::event_item_db::read_event_item_db(
        &main_path.join("CharacterInGame/EventItem.db"),
    )?;
    save_event_items(conn, &event_items)?;

    println!("Saving edit_items...");
    let edit_items = crate::references::edit_item_db::read_edit_item_db(
        &main_path.join("CharacterInGame/EditItem.db"),
    )?;
    save_edit_items(conn, &edit_items)?;

    println!("Saving party_level_db...");
    let party_levels = crate::references::party_level_db::read_party_level_db(
        &main_path.join("NpcInGame/PrtLevel.db"),
    )?;
    save_party_levels(conn, &party_levels)?;

    println!("Saving party_ini_db...");
    let party_inis =
        crate::references::party_ini_db::read_party_ini_db(&main_path.join("NpcInGame/PrtIni.db"))?;
    save_party_inis(conn, &party_inis)?;

    println!("Saving magic_spells...");
    let magic_spells =
        crate::references::magic_db::read_magic_db(&main_path.join("MagicInGame/Magic.db"))?;
    save_magic_spells(conn, &magic_spells)?;

    println!("Saving ch_datas...");
    let chdatas =
        crate::references::chdata_db::read_chdata(&main_path.join("CharacterInGame/ChData.db"))?;
    save_chdatas(conn, &chdatas)?;

    // Monsters are saved after magic_spells so their known-spell IDs can be
    // resolved against the already imported spell catalog.
    println!("Saving monsters...");
    let monsters = crate::references::monster_db::read_monster_db(
        &main_path.join("MonsterInGame/Monster.db"),
    )?;
    save_monsters(conn, &monsters)?;

    println!("Saving quests...");
    let quests =
        crate::references::quest_scr::read_quests(&main_path.join("ExtraInGame/Quest.scr"))?;
    save_quests(conn, &quests)?;

    println!("Saving messages...");
    let messages =
        crate::references::message_scr::read_messages(&main_path.join("ExtraInGame/Message.scr"))?;
    save_messages(conn, &messages)?;

    Ok(())
}

pub fn import_rest(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving party_refs...");
    let party_refs =
        crate::references::party_ref::read_part_refs(&main_path.join("Ref/PartyRef.ref"))?;
    save_party_refs(conn, &party_refs)?;
    println!("Saving draw_items...");
    let draw_items =
        crate::references::draw_item::read_draw_items(&main_path.join("Ref/DRAWITEM.ref"))?;
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
        let mut stmt = conn.prepare(insert_sql("npc_ref_files"))?;
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
        let npcrefs = crate::references::npc_ref::read_npc_ref(&main_path.join(npc_ref_file))?;
        // Resolve dialog_file_id from the map name shared between npc_ref_files
        // and dialogue_script_files (e.g. Npcmap1.ref ↔ Dlgmap1.dlg both → map1).
        let map_name = std::path::Path::new(npc_ref_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("Npc").or_else(|| s.strip_prefix("npc")))
            .map(|s| s.to_string());
        // A missing dialogue file is represented by id 0; real SQL errors are
        // propagated rather than silently treated as "no file".
        let dialog_file_id: i32 = match conn.query_row(
            "SELECT id FROM dialogue_script_files WHERE map_name = ?1",
            params![map_name],
            |row| row.get(0),
        ) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => 0,
            Err(err) => return Err(Box::new(err)),
        };
        save_npc_refs(conn, file_id as i32, dialog_file_id, &npcrefs)?;
    }

    println!("Saving event_npc_refs...");
    let event_npc_refs = crate::references::event_npc_ref::read_event_npc_ref(
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
        let mut stmt = conn.prepare(insert_sql("monster_ref_files"))?;
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
        let monster_refs =
            crate::references::monster_ref::read_monster_ref(&main_path.join(monster_ref_file))?;
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
        let mut stmt = conn.prepare(insert_sql("extra_ref_files"))?;
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
            crate::references::extra_ref::read_extra_ref(&main_path.join(extra_ref_file))?;
        save_extra_refs(conn, file_id as i32, &extra_refs)?;
    }
    Ok(())
}

pub fn import_event_scripts(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Saving event_scripts...");
    // Find all event script files in Ref directory
    let ref_dir = main_path.join("Ref");
    if ref_dir.exists() {
        let mut event_files = Vec::new();
        for entry in std::fs::read_dir(&ref_dir)? {
            let entry = entry?;
            if let Some(path) = entry.path().to_str()
                && (path.contains("Event") || path.contains("event"))
                && (path.ends_with(".scr") || path.ends_with(".SCR"))
            {
                event_files.push(entry.path());
            }
        }

        for event_file in event_files {
            let scripts = crate::references::event_scr::read_event_scripts(&event_file)?;
            save_event_scripts(conn, &scripts)?;
        }
    }
    Ok(())
}

pub fn import_sprite_files(main_path: &Path, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    use image::ImageEncoder;

    // Ensure the schema exists without dropping already imported data.
    super::create_all(conn)?;

    println!("Importing sprite files...");

    let file_insert_sql = insert_sql("sprite_files");
    let frame_insert_sql = insert_sql("sprite_frames");
    let seq_insert_sql = insert_sql("sprite_sequences");
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
        conn.execute(file_insert_sql, rusqlite::params![&normalized])?;
        let sprite_file_id: i64 =
            conn.query_row(id_query_sql, rusqlite::params![&normalized], |row| {
                row.get(0)
            })?;

        // 2. Prepare frame + sequence statements (per-file to avoid
        //    borrow conflicts with conn inside the loop body)
        let mut frame_stmt = conn.prepare(frame_insert_sql)?;
        let mut seq_stmt = conn.prepare(seq_insert_sql)?;

        match std::fs::File::open(&path) {
            Ok(file) => {
                let mut reader = std::io::BufReader::new(file);
                let infos = match crate::sprite::get_sprite_sequence_infos(&path) {
                    Ok(infos) => infos,
                    Err(e) => {
                        eprintln!("WARNING: sprite parse failed for {}: {e}", path.display());
                        errors += 1;
                        return Ok(());
                    }
                };

                for (seq_idx, info) in infos.iter().enumerate() {
                    let seq_idx = seq_idx as i32;

                    let frame_count = info.frame_infos.len() as i32;
                    let mut has_frame_data = false;

                    for (frame_idx, fi) in info.frame_infos.iter().enumerate() {
                        if fi.width <= 0 || fi.height <= 0 {
                            continue;
                        }
                        let img = match crate::sprite::render_frame_to_rgba_with_options(
                            &mut reader,
                            fi,
                            fi.width.unsigned_abs(),
                            fi.height.unsigned_abs(),
                            0,
                            0,
                            crate::sprite::SpriteRenderOptions::default(),
                        ) {
                            Ok(img) => img,
                            Err(e) => {
                                eprintln!(
                                    "WARNING: frame render failed for {}: {e}",
                                    path.display()
                                );
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
                                eprintln!(
                                    "WARNING: PNG encode failed for frame {} of {}",
                                    frame_idx,
                                    path.display()
                                );
                                errors += 1;
                                continue;
                            }
                        }

                        frame_stmt.execute(rusqlite::params![
                            sprite_file_id,
                            seq_idx,
                            frame_idx as i32,
                            &png_data,
                            fi.width,
                            fi.height,
                            fi.origin_x,
                            fi.origin_y,
                        ])?;
                        has_frame_data = true;
                    }

                    if has_frame_data {
                        let first = &info.frame_infos[0];
                        seq_stmt.execute(rusqlite::params![
                            sprite_file_id,
                            seq_idx,
                            frame_count,
                            first.width,
                            first.height,
                            first.origin_x,
                            first.origin_y,
                        ])?;
                    }
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
mod map_inventory_tests {
    use super::*;
    use crate::references::all_map_ini::Map as AllMap;
    use crate::references::enums::MapLighting;

    fn record(stem: &str) -> AllMap {
        AllMap {
            id: 1,
            map_filename: stem.to_owned(),
            map_name: "Test Map".to_owned(),
            pgp_filename: None,
            dlg_filename: None,
            lighting: MapLighting::Light,
        }
    }

    #[test]
    fn test_map_stems_follow_all_map_ini_order_and_strip_extension() {
        let dir = std::env::temp_dir().join(format!(
            "dispel-map-inventory-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp game dir");
        // WINDOWS-1250-compatible ASCII, with a comment and a CRLF line ending.
        std::fs::write(
            dir.join("AllMap.ini"),
            b"; id,file,name,pgp,dlg,lit\n1,cat1,Forest,null,null,0\r\n2,dun01.map,Dungeon,null,null,1\n",
        )
        .expect("write synthetic AllMap.ini");

        let inventory = read_map_inventory(&dir);
        assert_eq!(inventory.records.len(), 2);
        assert_eq!(
            inventory.stems,
            vec!["cat1".to_owned(), "dun01".to_owned()],
            "geometry import must follow the INI, including CRLF + .map suffixes"
        );

        std::fs::remove_dir_all(&dir).expect("remove temp game dir");
    }

    #[test]
    fn test_map_inventory_falls_back_only_when_ini_is_unparseable() {
        let dir = std::env::temp_dir().join(format!(
            "dispel-map-inventory-missing-{}",
            std::process::id()
        ));
        let inventory = read_map_inventory(&dir);
        assert!(inventory.records.is_empty());
        assert_eq!(inventory.stems, FALLBACK_MAP_STEMS);
    }

    #[test]
    fn test_normalize_map_stem_handles_bare_stem_and_suffix() {
        assert_eq!(normalize_map_stem(" cat1 "), "cat1");
        assert_eq!(normalize_map_stem("dun01.map"), "dun01");
        assert_eq!(normalize_map_stem("final.MAP"), "final");
    }

    #[test]
    fn test_stems_from_records_keep_ini_order() {
        let records = [record("map3"), record("cat1"), record("dun01.map")];
        let stems: Vec<String> = records
            .iter()
            .map(|m| normalize_map_stem(&m.map_filename))
            .collect();
        assert_eq!(stems, vec!["map3", "cat1", "dun01"]);
    }
}
