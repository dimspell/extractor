// use std::io::Write;
use std::collections::HashMap;

use crate::references::all_map_ini::Map;
use crate::references::dialog::Dialog;
use crate::references::draw_item::DrawItem;
use crate::references::edit_item_db::EditItem;
use crate::references::event_ini::Event;
use crate::references::event_item_db::EventItem;
use crate::references::event_npc_ref::EventNpcRef;
use crate::references::extra_ini::Extra;
use crate::references::extra_ref::ExtraRef;
use crate::references::heal_item_db::HealItem;
use crate::references::map_ini::MapIni;
use crate::references::misc_item_db::MiscItem;
use crate::references::monster_db::Monster;
use crate::references::monster_ini::MonsterIni;
use crate::references::monster_ref::MonsterRef;
use crate::references::npc_ini::NpcIni;
use crate::references::npc_ref::NPC;
use crate::references::party_pgp::PartyPgp;
use crate::references::party_ref::PartyRef;
use crate::references::store_db::{Store, StoreProduct};
use crate::references::wave_ini::WaveIni;
use crate::references::weapons_db::WeaponItem;
use rusqlite::{params, Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    let tables = vec![
        "dialogs",
        "draw_items",
        "edit_items",
        "event_items",
        "event_npc_refs",
        "events",
        "extra_refs",
        "extras",
        "heal_items",
        "map_inis",
        "map_objects",
        "map_sprites",
        "map_tiles",
        "maps",
        "misc_items",
        "monster_inis",
        "monster_refs",
        "monsters",
        "npc_inis",
        "npc_refs",
        "party_pgps",
        "party_refs",
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

    Ok(())
}

pub fn save_npc_refs(conn: &Connection, file_path: &str, npc_refs: &Vec<NPC>) -> Result<()> {
    for npc in npc_refs {
        add_npc_ref(conn, file_path, npc)?;
    }
    Ok(())
}

fn add_npc_ref(conn: &Connection, file_path: &str, npc: &NPC) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_npc_ref.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_monster_refs(
    conn: &Connection,
    file_path: &str,
    monster_refs: &Vec<MonsterRef>,
) -> Result<()> {
    for monster_ref in monster_refs {
        add_monster_ref(conn, file_path, monster_ref)?;
    }
    Ok(())
}

fn add_monster_ref(conn: &Connection, file_path: &str, monster_ref: &MonsterRef) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_monster_ref.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_extra_refs(
    conn: &Connection,
    file_path: &str,
    extra_refs: &Vec<ExtraRef>,
) -> Result<()> {
    for extra_ref in extra_refs {
        add_extra_ref(conn, file_path, extra_ref)?;
    }
    Ok(())
}

fn add_extra_ref(conn: &Connection, file_path: &str, extra_ref: &ExtraRef) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_extra_ref.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_weapons(conn: &Connection, weapons: &Vec<WeaponItem>) -> Result<()> {
    for weapon in weapons {
        add_weapon(conn, weapon)?;
    }
    Ok(())
}

fn add_weapon(conn: &Connection, weapon: &WeaponItem) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_weapon.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_edit_items(conn: &Connection, edit_items: &Vec<EditItem>) -> Result<()> {
    for item in edit_items {
        add_edit_item(conn, item)?;
    }
    Ok(())
}

fn add_edit_item(conn: &Connection, item: &EditItem) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_edit_item.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_event_items(conn: &Connection, event_items: &Vec<EventItem>) -> Result<()> {
    for item in event_items {
        add_event_item(conn, item)?;
    }
    Ok(())
}

fn add_event_item(conn: &Connection, item: &EventItem) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_event_item.sql"),
        params![item.id, item.name, item.description,],
    )?;
    Ok(())
}

pub fn save_event_npc_refs(conn: &Connection, npc_refs: &Vec<EventNpcRef>) -> Result<()> {
    for npc in npc_refs {
        add_event_npc_ref(conn, npc)?;
    }
    Ok(())
}

fn add_event_npc_ref(conn: &Connection, npc: &EventNpcRef) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_event_npc_ref.sql"),
        params![npc.id, npc.event_id, npc.name],
    )?;
    Ok(())
}

pub fn save_misc_items(conn: &Connection, misc_items: &Vec<MiscItem>) -> Result<()> {
    for item in misc_items {
        add_misc_item(conn, item)?;
    }
    Ok(())
}

fn add_misc_item(conn: &Connection, item: &MiscItem) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_misc_item.sql"),
        params![item.id, item.name, item.description, item.base_price],
    )?;
    Ok(())
}

pub fn save_heal_items(conn: &Connection, heal_items: &Vec<HealItem>) -> Result<()> {
    for item in heal_items {
        add_heal_item(conn, item)?;
    }
    Ok(())
}

fn add_heal_item(conn: &Connection, item: &HealItem) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_heal_item.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_stores(conn: &Connection, stores: &Vec<Store>) -> Result<()> {
    for store in stores {
        add_store(conn, store)?;
        for product in &store.products {
            add_store_product(conn, store, product)?;
        }
    }
    Ok(())
}

fn add_store(conn: &Connection, store: &Store) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_store.sql"),
        params![
            store.index,
            store.store_name,
            store.inn_night_cost,
            store.some_unknown_number,
            store.invitation,
            store.haggle_success,
            store.haggle_fail,
        ],
    )?;
    Ok(())
}

fn add_store_product(conn: &Connection, store: &Store, store_product: &StoreProduct) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_store_product.sql"),
        params![
            store.index,
            store_product.0,
            store_product.1,
            store_product.2,
        ],
    )?;
    Ok(())
}

pub fn save_monsters(conn: &Connection, monsters: &Vec<Monster>) -> Result<()> {
    for monster in monsters {
        add_monster(conn, monster)?;
    }
    Ok(())
}

fn add_monster(conn: &Connection, monster: &Monster) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_monster.sql"),
        params![
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
        ],
    )?;
    Ok(())
}

pub fn save_maps(conn: &Connection, maps: &Vec<Map>) -> Result<()> {
    for map in maps {
        add_map(conn, map)?;
    }
    Ok(())
}

fn add_map(conn: &Connection, map: &Map) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_map.sql"),
        params![
            map.id,
            map.map_filename,
            map.map_name,
            map.pgp_filename,
            map.dlg_filename,
            map.is_light,
        ],
    )?;
    Ok(())
}

pub fn save_events(conn: &Connection, events: &Vec<Event>) -> Result<()> {
    for event in events {
        add_event(conn, event)?;
    }
    Ok(())
}

fn add_event(conn: &Connection, event: &Event) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_event.sql"),
        params![
            event.event_id,
            event.previous_event_id,
            event.event_type_id,
            event.event_filename,
            event.counter,
        ],
    )?;
    Ok(())
}

pub fn save_extras(conn: &Connection, extras: &Vec<Extra>) -> Result<()> {
    for extra in extras {
        add_extra(conn, extra)?;
    }
    Ok(())
}

fn add_extra(conn: &Connection, extra: &Extra) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_extra.sql"),
        params![
            extra.id,
            extra.sprite_filename,
            extra.unknown,
            extra.description,
        ],
    )?;
    Ok(())
}

pub fn save_monster_inis(conn: &Connection, monster_inis: &Vec<MonsterIni>) -> Result<()> {
    for monster_ini in monster_inis {
        add_monster_ini(conn, monster_ini)?;
    }
    Ok(())
}

fn add_monster_ini(conn: &Connection, monster_ini: &MonsterIni) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_monster_ini.sql"),
        params![
            monster_ini.id,
            monster_ini.name,
            monster_ini.sprite_filename,
            monster_ini.attack,
            monster_ini.hit,
            monster_ini.death,
            monster_ini.walking,
            monster_ini.casting_magic,
        ],
    )?;
    Ok(())
}

pub fn save_npc_inis(conn: &Connection, npc_inis: &Vec<NpcIni>) -> Result<()> {
    for npc_ini in npc_inis {
        add_npc_ini(conn, npc_ini)?;
    }
    Ok(())
}

fn add_npc_ini(conn: &Connection, npc_ini: &NpcIni) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_npc_ini.sql"),
        params![npc_ini.id, npc_ini.sprite_filename, npc_ini.description,],
    )?;
    Ok(())
}

pub fn save_wave_inis(conn: &Connection, wave_inis: &Vec<WaveIni>) -> Result<()> {
    for wave_ini in wave_inis {
        add_wave_ini(conn, wave_ini)?;
    }
    Ok(())
}

fn add_wave_ini(conn: &Connection, wave_ini: &WaveIni) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_wave_ini.sql"),
        params![wave_ini.id, wave_ini.snf_filename, wave_ini.unknown_flag,],
    )?;
    Ok(())
}

pub fn save_map_inis(conn: &Connection, map_inis: &Vec<MapIni>) -> Result<()> {
    for map_ini in map_inis {
        add_map_ini(conn, map_ini)?;
    }
    Ok(())
}

fn add_map_ini(conn: &Connection, map_ini: &MapIni) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_map_ini.sql"),
        params![
            map_ini.id,
            map_ini.event_id_on_camera_move,
            map_ini.start_pos_x,
            map_ini.start_pos_y,
            map_ini.map_id,
            map_ini.monsters_filename,
            map_ini.npc_filename,
            map_ini.extra_filename,
            map_ini.cd_music_track_number,
        ],
    )?;
    Ok(())
}

pub fn save_party_refs(conn: &Connection, party_refs: &Vec<PartyRef>) -> Result<()> {
    for party_ref in party_refs {
        add_party_ref(conn, party_ref)?;
    }
    Ok(())
}

fn add_party_ref(conn: &Connection, party_ref: &PartyRef) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_party_ref.sql"),
        params![
            party_ref.id,
            party_ref.full_name,
            party_ref.job_name,
            party_ref.root_map_id,
            party_ref.npc_id,
            party_ref.dlg_when_not_in_party,
            party_ref.dlg_when_in_party,
            party_ref.ghost_face_id,
        ],
    )?;
    Ok(())
}

pub fn save_draw_items(conn: &Connection, draw_items: &Vec<DrawItem>) -> Result<()> {
    for draw_item in draw_items {
        add_draw_item(conn, draw_item)?;
    }
    Ok(())
}

fn add_draw_item(conn: &Connection, draw_item: &DrawItem) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_draw_item.sql"),
        params![
            draw_item.map_id,
            draw_item.x_coord,
            draw_item.y_coord,
            draw_item.item_id,
        ],
    )?;
    Ok(())
}

pub fn save_party_pgps(conn: &Connection, party_pgps: &Vec<PartyPgp>) -> Result<()> {
    for party_pgp in party_pgps {
        add_party_pgp(conn, party_pgp)?;
    }
    Ok(())
}

fn add_party_pgp(conn: &Connection, party_pgp: &PartyPgp) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_party_pgp.sql"),
        params![
            party_pgp.id,
            party_pgp.dialog_text,
            party_pgp.unknown_id1,
            party_pgp.unknown_id2,
        ],
    )?;
    Ok(())
}

pub fn save_dialogs(conn: &Connection, dialog_file: &str, dialogs: &Vec<Dialog>) -> Result<()> {
    for dialog in dialogs {
        add_dialog(conn, dialog_file, dialog)?;
    }
    Ok(())
}

fn add_dialog(conn: &Connection, dialog_file: &str, dialog: &Dialog) -> Result<()> {
    conn.execute(
        include_str!("queries/insert_dialog.sql"),
        params![
            dialog_file,
            dialog.id,
            dialog.previous_event_id,
            dialog.next_dialog_to_check,
            dialog.dialog_type_id,
            dialog.dialog_owner,
            dialog.dialog_id,
            dialog.event_id,
        ],
    )?;
    Ok(())
}
pub fn save_map_tiles(
    conn: &Connection,
    map_id: &str,
    gtl_tiles: &HashMap<crate::map::Coords, i32>,
    btl_tiles: &HashMap<crate::map::Coords, i32>,
    collisions: &HashMap<crate::map::Coords, bool>,
    events: &HashMap<crate::map::Coords, crate::map::EventBlock>,
    width: i32,
    height: i32,
) -> Result<()> {
    let mut stmt = conn.prepare(include_str!("queries/insert_map_tile.sql"))?;

    let offset_x = width / 2;
    let offset_y = height / 2;

    println!(
        "Inserting map tiles for map {}, width {}, height {}",
        map_id, width, height
    );

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

    Ok(())
}

pub fn save_map_objects(
    conn: &Connection,
    map_id: &str,
    tiled_infos: &Vec<crate::map::TiledObjectInfo>,
) -> Result<()> {
    let mut stmt = conn.prepare(include_str!("queries/insert_map_object.sql"))?;

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

    Ok(())
}

pub fn save_map_sprites(
    conn: &Connection,
    map_id: &str,
    sprite_blocks: &Vec<crate::map::SpriteInfoBlock>,
) -> Result<()> {
    let mut stmt = conn.prepare(include_str!("queries/insert_map_sprite.sql"))?;

    for (sprite_idx, block) in sprite_blocks.iter().enumerate() {
        stmt.execute(params![
            map_id,
            sprite_idx as i32,
            block.sprite_x,
            block.sprite_y,
            block.sprite_id as i32,
        ])?;
    }

    Ok(())
}
