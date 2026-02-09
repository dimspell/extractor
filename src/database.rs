// use std::io::Write;
use std::collections::HashMap;

use crate::references::all_map_ini::Map;
use crate::references::dialog::Dialog;
use crate::references::dialogue_text::DialogueText;
use crate::references::draw_item::DrawItem;
use crate::references::edit_item_db::EditItem;
use crate::references::event_ini::Event;
use crate::references::event_item_db::EventItem;
use crate::references::event_npc_ref::EventNpcRef;
use crate::references::extra_ini::Extra;
use crate::references::extra_ref::ExtraRef;
use crate::references::heal_item_db::HealItem;
use crate::references::magic_db::MagicSpell;
use crate::references::map_ini::MapIni;
use crate::references::message_scr::Message;
use crate::references::misc_item_db::MiscItem;
use crate::references::monster_db::Monster;
use crate::references::monster_ini::MonsterIni;
use crate::references::monster_ref::MonsterRef;
use crate::references::npc_ini::NpcIni;
use crate::references::npc_ref::NPC;
use crate::references::party_ini_db::PartyIniNpc;
use crate::references::party_level_db::PartyLevelNpc;
use crate::references::party_pgp::PartyPgp;
use crate::references::party_ref::PartyRef;
use crate::references::quest_scr::Quest;
use crate::references::store_db::Store;
use crate::references::wave_ini::WaveIni;
use crate::references::weapons_db::WeaponItem;
use rusqlite::{params, Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    // Optimization PRAGMAs
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 30000000000;",
    )?;

    let tables = vec![
        "dialogue_texts",
        "dialogs",
        "draw_items",
        "edit_items",
        "event_items",
        "event_npc_refs",
        "events",
        "extra_refs",
        "extras",
        "heal_items",
        "magic_spells",
        "map_inis",
        "map_metadata",
        "map_objects",
        "map_sprites",
        "map_tiles",
        "maps",
        "messages",
        "misc_items",
        "monster_inis",
        "monster_refs",
        "monsters",
        "npc_inis",
        "npc_refs",
        "party_inis",
        "party_levels",
        "party_pgps",
        "party_refs",
        "quests",
        "store_products",
        "stores",
        "wave_inis",
        "weapons",
    ];

    for table in tables {
        conn.execute(&format!("DROP TABLE IF EXISTS {}", table), [])?;
    }

    conn.execute_batch(include_str!("queries/create_table_npc_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monster_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_extra_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_weapons.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_edit_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_npc_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_misc_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_heal_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_stores.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_store_products.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monsters.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_maps.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_events.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_extras.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monster_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_npc_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_wave_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_draw_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_pgps.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_dialogs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_tiles.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_objects.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_sprites.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_metadata.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_dialogue_texts.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_levels.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_magic_spells.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_quests.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_messages.sql"))?;

    Ok(())
}

pub fn save_npc_refs(conn: &mut Connection, file_path: &str, npc_refs: &Vec<NPC>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_npc_ref.sql"))?;
        for npc in npc_refs {
            stmt.execute(params![
                file_path,
                npc.index,
                npc.id,
                npc.npc_id,
                npc.name,
                npc.party_script_id,
                npc.show_on_event,
                npc.goto1_filled,
                npc.goto2_filled,
                npc.goto3_filled,
                npc.goto4_filled,
                npc.goto1_x,
                npc.goto2_x,
                npc.goto3_x,
                npc.goto4_x,
                npc.goto1_y,
                npc.goto2_y,
                npc.goto3_y,
                npc.goto4_y,
                npc.looking_direction,
                npc.dialog_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_npc_ref removed as it is now inlined for performance

pub fn save_monster_refs(
    conn: &mut Connection,
    file_path: &str,
    monster_refs: &Vec<MonsterRef>,
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_monster_ref.sql"))?;
        for monster_ref in monster_refs {
            stmt.execute(params![
                file_path,
                monster_ref.index,
                monster_ref.file_id,
                monster_ref.mon_id,
                monster_ref.pos_x,
                monster_ref.pos_y,
                monster_ref.loot1_item_id,
                monster_ref.loot1_item_type,
                monster_ref.loot2_item_id,
                monster_ref.loot2_item_type,
                monster_ref.loot3_item_id,
                monster_ref.loot3_item_type,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_monster_ref removed

pub fn save_extra_refs(
    conn: &mut Connection,
    file_path: &str,
    extra_refs: &Vec<ExtraRef>,
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_extra_ref.sql"))?;
        for extra_ref in extra_refs {
            stmt.execute(params![
                file_path,
                extra_ref.id,
                extra_ref.number_in_file,
                extra_ref.ext_id,
                extra_ref.name,
                extra_ref.object_type,
                extra_ref.x_pos,
                extra_ref.y_pos,
                extra_ref.rotation,
                extra_ref.closed,
                extra_ref.required_item_id,
                extra_ref.required_item_type_id,
                extra_ref.required_item_id2,
                extra_ref.required_item_type_id2,
                extra_ref.gold_amount,
                extra_ref.item_id,
                extra_ref.item_type_id,
                extra_ref.item_count,
                extra_ref.event_id,
                extra_ref.message_id,
                extra_ref.visibility,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_extra_ref removed

pub fn save_weapons(conn: &mut Connection, weapons: &Vec<WeaponItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_weapon.sql"))?;
        for weapon in weapons {
            stmt.execute(params![
                weapon.id,
                weapon.name,
                weapon.description,
                weapon.base_price,
                weapon.health_points,
                weapon.magic_points,
                weapon.strength,
                weapon.agility,
                weapon.wisdom,
                weapon.tf,
                weapon.unk,
                weapon.trf,
                weapon.attack,
                weapon.defense,
                weapon.mag,
                weapon.durability,
                weapon.req_strength,
                weapon.req_zw,
                weapon.req_wisdom,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_weapon removed

pub fn save_edit_items(conn: &mut Connection, edit_items: &Vec<EditItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_edit_item.sql"))?;
        for item in edit_items {
            stmt.execute(params![
                item.index,
                item.name,
                item.description,
                item.base_price,
                item.health_points,
                item.magic_points,
                item.strength,
                item.agility,
                item.wisdom,
                item.to_hit,
                item.to_dodge,
                item.to_hit,
                item.offense,
                item.defense,
                item.item_destroying_power,
                item.modifies_item,
                item.additional_effect,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_event_items(conn: &mut Connection, event_items: &Vec<EventItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_event_item.sql"))?;
        for item in event_items {
            stmt.execute(params![item.id, item.name, item.description,])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_event_item removed

pub fn save_event_npc_refs(conn: &mut Connection, npc_refs: &Vec<EventNpcRef>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_event_npc_ref.sql"))?;
        for npc in npc_refs {
            stmt.execute(params![npc.id, npc.event_id, npc.name])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_event_npc_ref removed

pub fn save_misc_items(conn: &mut Connection, misc_items: &Vec<MiscItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_misc_item.sql"))?;
        for item in misc_items {
            stmt.execute(params![
                item.id,
                item.name,
                item.description,
                item.base_price
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_misc_item removed

pub fn save_heal_items(conn: &mut Connection, heal_items: &Vec<HealItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_heal_item.sql"))?;
        for item in heal_items {
            stmt.execute(params![
                item.id,
                item.name,
                item.description,
                item.base_price,
                item.pz,
                item.pm,
                item.full_pz,
                item.full_pm,
                item.poison_heal,
                item.petrif_heal,
                item.polimorph_heal,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_heal_item removed

pub fn save_stores(conn: &mut Connection, stores: &Vec<Store>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt_store = tx.prepare(include_str!("queries/insert_store.sql"))?;
        let mut stmt_product = tx.prepare(include_str!("queries/insert_store_product.sql"))?;

        for store in stores {
            stmt_store.execute(params![
                store.index,
                store.store_name,
                store.inn_night_cost,
                store.some_unknown_number,
                store.invitation,
                store.haggle_success,
                store.haggle_fail,
            ])?;

            for product in &store.products {
                stmt_product.execute(params![store.index, product.0, product.1, product.2,])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

// add_store and add_store_product removed

pub fn save_monsters(conn: &mut Connection, monsters: &Vec<Monster>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_monster.sql"))?;
        for monster in monsters {
            stmt.execute(params![
                monster.id,
                monster.name,
                monster.health_points_max,
                monster.health_points_min,
                monster.magic_points_max,
                monster.magic_points_min,
                monster.walk_speed,
                monster.to_hit_max,
                monster.to_hit_min,
                monster.to_dodge_max,
                monster.to_dodge_min,
                monster.offense_max,
                monster.offense_min,
                monster.defense_max,
                monster.defense_min,
                monster.magic_attack_max,
                monster.magic_attack_min,
                monster.is_undead,
                monster.has_blood,
                monster.ai_type,
                monster.exp_gain_max,
                monster.exp_gain_min,
                monster.gold_drop_max,
                monster.gold_drop_min,
                monster.detection_sight_size,
                monster.distance_range_size,
                monster.known_spell_slot1,
                monster.known_spell_slot2,
                monster.known_spell_slot3,
                monster.is_oversize,
                monster.magic_level,
                monster.special_attack,
                monster.special_attack_chance,
                monster.special_attack_duration,
                monster.boldness,
                monster.attack_speed,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_monster removed

pub fn save_maps(conn: &mut Connection, maps: &Vec<Map>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_map.sql"))?;
        for map in maps {
            stmt.execute(params![
                map.id,
                map.map_filename,
                map.map_name,
                map.pgp_filename,
                map.dlg_filename,
                map.is_light,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_map removed

pub fn save_map_metadata(
    conn: &mut Connection,
    map_id: &str,
    model: &crate::map::MapModel,
) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_map_metadata.sql"),
        params![
            map_id,
            model.tiled_map_width,
            model.tiled_map_height,
            model.map_width_in_pixels,
            model.map_height_in_pixels,
            model.map_non_occluded_start_x,
            model.map_non_occluded_start_y,
            model.occluded_map_in_pixels_width,
            model.occluded_map_in_pixels_height,
        ],
    )?;

    Ok(())
}

pub fn save_dialogue_texts(
    conn: &mut Connection,
    file_name: &str,
    texts: &Vec<DialogueText>,
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_dialogue_text.sql"))?;
        for text in texts {
            stmt.execute(params![
                file_name,
                text.id,
                text.text,
                text.comment,
                text.param1,
                text.param2,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_dialogue_text removed

pub fn save_events(conn: &mut Connection, events: &Vec<Event>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_event.sql"))?;
        for event in events {
            stmt.execute(params![
                event.event_id,
                event.previous_event_id,
                event.event_type_id,
                event.event_filename,
                event.counter,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_event removed

pub fn save_extras(conn: &mut Connection, extras: &Vec<Extra>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_extra.sql"))?;
        for extra in extras {
            stmt.execute(params![
                extra.id,
                extra.sprite_filename,
                extra.unknown,
                extra.description,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_extra removed

pub fn save_monster_inis(conn: &mut Connection, monster_inis: &Vec<MonsterIni>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_monster_ini.sql"))?;
        for monster_ini in monster_inis {
            stmt.execute(params![
                monster_ini.id,
                monster_ini.name,
                monster_ini.sprite_filename,
                monster_ini.attack,
                monster_ini.hit,
                monster_ini.death,
                monster_ini.walking,
                monster_ini.casting_magic,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_monster_ini removed

pub fn save_npc_inis(conn: &mut Connection, npc_inis: &Vec<NpcIni>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_npc_ini.sql"))?;
        for npc_ini in npc_inis {
            stmt.execute(params![
                npc_ini.id,
                npc_ini.sprite_filename,
                npc_ini.description,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_npc_ini removed

pub fn save_wave_inis(conn: &mut Connection, wave_inis: &Vec<WaveIni>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_wave_ini.sql"))?;
        for wave_ini in wave_inis {
            stmt.execute(params![
                wave_ini.id,
                wave_ini.snf_filename,
                wave_ini.unknown_flag,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_wave_ini removed

pub fn save_map_inis(conn: &mut Connection, map_inis: &Vec<MapIni>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_map_ini.sql"))?;
        for map_ini in map_inis {
            stmt.execute(params![
                map_ini.id,
                map_ini.event_id_on_camera_move,
                map_ini.start_pos_x,
                map_ini.start_pos_y,
                map_ini.map_id,
                map_ini.monsters_filename,
                map_ini.npc_filename,
                map_ini.extra_filename,
                map_ini.cd_music_track_number,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_map_ini removed

pub fn save_party_refs(conn: &mut Connection, party_refs: &Vec<PartyRef>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_party_ref.sql"))?;
        for party_ref in party_refs {
            stmt.execute(params![
                party_ref.id,
                party_ref.full_name,
                party_ref.job_name,
                party_ref.root_map_id,
                party_ref.npc_id,
                party_ref.dlg_when_not_in_party,
                party_ref.dlg_when_in_party,
                party_ref.ghost_face_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_party_ref removed

pub fn save_draw_items(conn: &mut Connection, draw_items: &Vec<DrawItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_draw_item.sql"))?;
        for draw_item in draw_items {
            stmt.execute(params![
                draw_item.map_id,
                draw_item.x_coord,
                draw_item.y_coord,
                draw_item.item_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_draw_item removed

pub fn save_party_pgps(conn: &mut Connection, party_pgps: &Vec<PartyPgp>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_party_pgp.sql"))?;
        for party_pgp in party_pgps {
            stmt.execute(params![
                party_pgp.id,
                party_pgp.dialog_text,
                party_pgp.unknown_id1,
                party_pgp.unknown_id2,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_party_pgp removed

pub fn save_dialogs(conn: &mut Connection, dialog_file: &str, dialogs: &Vec<Dialog>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_dialog.sql"))?;
        for dialog in dialogs {
            stmt.execute(params![
                dialog_file,
                dialog.id,
                dialog.previous_event_id,
                dialog.next_dialog_to_check,
                dialog.dialog_type_id,
                dialog.dialog_owner,
                dialog.dialog_id,
                dialog.event_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// add_dialog removed
pub fn save_map_tiles(
    conn: &mut Connection,
    map_id: &str,
    gtl_tiles: &HashMap<crate::map::Coords, i32>,
    btl_tiles: &HashMap<crate::map::Coords, i32>,
    collisions: &HashMap<crate::map::Coords, bool>,
    events: &HashMap<crate::map::Coords, crate::map::EventBlock>,
    width: i32,
    height: i32,
) -> Result<()> {
    let tx = conn.transaction()?;

    let offset_x = width / 2;
    let offset_y = height / 2;

    println!(
        "Inserting map tiles for map {}, width {}, height {}",
        map_id, width, height
    );

    {
        let mut stmt = tx.prepare(include_str!("queries/insert_map_tile.sql"))?;

        for y in 0..height {
            for x in 0..width {
                let coords = (x, y);
                let gtl_id = gtl_tiles.get(&coords).cloned().unwrap_or(0);
                let btl_id = btl_tiles.get(&coords).cloned().unwrap_or(0);
                let collision = collisions.get(&coords).cloned().unwrap_or(false);
                let event_id = events.get(&coords).map(|e| e.event_id).unwrap_or(0);

                if gtl_id == 0 && btl_id == 0 && !collision && event_id == 0 {
                    continue;
                }

                stmt.execute(params![
                    map_id,
                    x - offset_x,
                    y - offset_y,
                    gtl_id,
                    btl_id,
                    collision,
                    event_id as i32,
                ])?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn save_map_objects(
    conn: &mut Connection,
    map_id: &str,
    tiled_infos: &Vec<crate::map::TiledObjectInfo>,
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_map_object.sql"))?;
        for (obj_idx, info) in tiled_infos.iter().enumerate() {
            for (stack_order, btl_id) in info.ids.iter().enumerate() {
                stmt.execute(params![
                    map_id,
                    obj_idx as i32,
                    info.x,
                    info.y,
                    *btl_id as i32,
                    stack_order as i32,
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_map_sprites(
    conn: &mut Connection,
    map_id: &str,
    sprite_blocks: &Vec<crate::map::SpriteInfoBlock>,
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_map_sprite.sql"))?;
        for (sprite_idx, block) in sprite_blocks.iter().enumerate() {
            stmt.execute(params![
                map_id,
                sprite_idx as i32,
                block.sprite_x,
                block.sprite_y,
                block.sprite_id as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_party_levels(conn: &mut Connection, npcs: &Vec<PartyLevelNpc>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_party_level.sql"))?;
        for npc in npcs {
            for record in &npc.records {
                stmt.execute(params![
                    npc.npc_index as i32,
                    record.level as i32,
                    record.strength as i32,
                    record.constitution as i32,
                    record.wisdom as i32,
                    record.health_points as i32,
                    record.magic_points as i32,
                    record.agility as i32,
                    record.attack as i32,
                    record.mana_recharge as i32,
                    record.defense as i32,
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_party_inis(conn: &mut Connection, npcs: &Vec<PartyIniNpc>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_party_ini.sql"))?;
        for (idx, npc) in npcs.iter().enumerate() {
            stmt.execute(params![
                idx as i32,
                npc.name,
                npc.flags as i32,
                npc.kind as i32,
                npc.value as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_magic_spells(conn: &mut Connection, spells: &Vec<MagicSpell>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_magic_spell.sql"))?;
        for spell in spells {
            stmt.execute(params![
                spell.id,
                spell.enabled,
                spell.flag1,
                spell.mana_cost,
                spell.success_rate,
                spell.base_damage,
                spell.reserved1,
                spell.reserved2,
                spell.flag2,
                spell.range,
                spell.reserved3,
                spell.level_required,
                spell.constant1,
                spell.effect_value,
                spell.effect_type,
                spell.effect_modifier,
                spell.reserved4,
                spell.magic_school,
                spell.flag3,
                spell.animation_id,
                spell.visual_id,
                spell.icon_id,
                spell.target_type,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_quests(conn: &mut Connection, quests: &Vec<Quest>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_quest.sql"))?;
        for quest in quests {
            stmt.execute(params![
                quest.id,
                quest.type_id,
                quest.title,
                quest.description,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_messages(conn: &mut Connection, messages: &Vec<Message>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("queries/insert_message.sql"))?;
        for message in messages {
            stmt.execute(params![
                message.id,
                message.line1,
                message.line2,
                message.line3,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
