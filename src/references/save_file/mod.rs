// Save file extraction and parsing for Dispel RPG
//
// This module provides comprehensive parsing of Dispel RPG save files (.sav)

pub mod character;
pub mod events;
pub mod game_tmp;
pub mod inventory;
pub mod journal;
pub mod map_viewport;
pub mod tests;

use super::extractor::{Extractor, read_null_terminated_windows_1250};
pub use crate::references::save_file::character::{
    BeltPotionSlot, CharacterDataHeader, CharacterStatsHeader, EquipmentSlot,
    InventoryPlacementEntry, LearnedSpells, PartyMember,
};
pub use crate::references::save_file::character::{CharacterIdentity, CharacterStats};
pub use crate::references::save_file::events::{EventRecord, PostEventsData};
pub use crate::references::save_file::game_tmp::{
    DrawItemEditItem, DrawItemEventItem, DrawItemHealItem, DrawItemMiscItem, DrawItemWeaponItem,
    ExtraObjectRecord, ExtraObjectTrailerRecord, MapExtraObjectsTrailer, MapSectionData,
    MonsterRecord, NpcRecord,
};
pub use crate::references::save_file::inventory::{
    InventoryData, InventoryEditItem, InventoryEventItem, InventoryHealItem, InventoryMiscItem,
    InventoryWeaponItem,
};
pub use crate::references::save_file::journal::{JournalData, JournalEntry, JournalHeader};
pub use crate::references::save_file::map_viewport::{
    MapViewportCell, MapViewportState, PostMapsData,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
// use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

/// Complete save file structure.
///
/// More fields will be added in future phases.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFile {
    /// Jump address after all map data (first 4 bytes of the file).
    /// The maps section is followed by alignment to this address.
    pub game_tmp_blob_size: u32,
    /// Per-map world state (game.tmp blob length-prefixed by [Self.game_tmp_blob_size]).
    pub maps: Vec<MapSectionData>,
    /// Unknown data between maps and sprite paths (header + variable-size remainder).
    pub post_maps: PostMapsData,
    /// Character sprite paths (4 × 60-byte WINDOWS-1250 strings).
    pub sprite_paths: Vec<String>,
    /// Unknown 8 bytes
    pub unknown_before_stats: Vec<u8>,
    pub character_position_x: i16,
    pub character_position_y: i16,
    /// Header before character stats, including the currently selected spell.
    pub character_stats_header: CharacterStatsHeader,
    /// Parsed character stats (core, combat, skills, weapon skills).
    pub character_stats: CharacterStats,
    /// Unknown bytes after stats block (9 bytes).
    pub unknown_after_stats: Vec<u8>,
    /// Raw inventory data (5 item categories).
    pub inventory: InventoryData,
    /// Character identity (name, class, unknown blocks).
    pub character_identity: CharacterIdentity,
    /// Event scripts (2251 × 284 bytes).
    pub events: Vec<EventRecord>,
    /// Unknown data between events and journal (3 sub-blocks).
    pub post_events: PostEventsData,
    /// Journal entries (main, side, trade — 100 entries each).
    pub journal: JournalData,
}

impl SaveFile {
    /// Parse complete save file from binary data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // ── 1. HEADER (4 bytes) ──
        let jump_addr_after_maps = reader.read_u32::<LittleEndian>()? as usize;

        // ── 2. Maps ──
        let number_of_visited_map = reader.read_u32::<LittleEndian>()?;
        let maps = Self::parse_maps_section(&mut reader, number_of_visited_map)?;

        if jump_addr_after_maps != reader.position() as usize {
            eprintln!(
                "jump_addr_after_maps ({:?}) != reader.position() {:?}",
                jump_addr_after_maps,
                reader.position() as usize
            );

            reader.set_position(jump_addr_after_maps as u64);
        }

        // ── 3. Unknown data between maps and sprite paths ──
        let post_maps = Self::parse_post_maps_data(&mut reader, number_of_visited_map)?;

        // ── 4. Character sprite paths (4 × 60-byte WINDOWS-1250 strings) ──
        let sprite_paths = Self::parse_sprite_paths(&mut reader)?;

        // ── 5 Character stats ──
        let (
            unknown_before_stats,
            character_position_x,
            character_position_y,
            character_stats_header,
            character_stats,
            unknown_after_stats,
        ) = Self::parse_character_stats(&mut reader)?;

        // ── 6. Inventory (5 categories, each count-prefixed) ──
        let inventory = Self::parse_inventory_section(&mut reader)?;

        // ── 7. Character identity (unknown block + name + class + large unknown) ──
        let character_identity = Self::parse_character_identity(&mut reader)?;

        // ── 8. Events (2251 × 284 bytes) ──
        let events = Self::parse_events_section(&mut reader)?;

        // ── 9. Unknown data between events and journal ──
        let post_events = Self::parse_post_events_data(&mut reader)?;

        // ── 10. Journal (42-byte header + 3 sections × 100 × 37 bytes) ──
        let journal = Self::parse_journal_section(&mut reader)?;

        Ok(SaveFile {
            game_tmp_blob_size: jump_addr_after_maps as u32,
            maps,
            post_maps,
            sprite_paths,
            unknown_before_stats,
            character_position_x,
            character_position_y,
            character_stats_header,
            character_stats,
            unknown_after_stats,
            inventory,
            character_identity,
            events,
            post_events,
            journal,
        })
    }

    /// Generic count-prefixed item section reader.
    ///
    /// Each section is stored as `[count: u16][count × record_size bytes]`.
    /// Parses each record via the provided `parse` function.
    fn read_item_section<R: Read, T>(
        reader: &mut R,
        record_size: usize,
        parse: fn(&[u8]) -> std::io::Result<T>,
    ) -> std::io::Result<Vec<T>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * record_size];
        reader.read_exact(&mut data)?;

        data.chunks_exact(record_size).map(parse).collect()
    }

    /// Parse all map sections from the reader.
    ///
    /// Each map has:
    ///   `[map_id: u32][monsters][npcs][sep: u32][extra_objects][trailer]
    ///    [draw_items_weapon][draw_items_heal][draw_items_edit]
    ///    [draw_items_misc][draw_items_event][end_sep: u32]`
    fn parse_maps_section<R: Read + Seek>(
        reader: &mut R,
        map_count: u32,
    ) -> std::io::Result<Vec<MapSectionData>> {
        let mut maps = Vec::with_capacity(map_count as usize);

        for _ in 0..map_count {
            let map_id = reader.read_u32::<LittleEndian>()?;

            // ── 2.1. Monsters ──
            let monster_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut monsters_data = vec![0u8; monster_count * 329];
            reader.read_exact(&mut monsters_data)?;
            let monsters = monsters_data
                .chunks_exact(329)
                .map(MonsterRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.2. NPCs ──
            let npc_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut npcs_data = vec![0u8; npc_count * 349];
            reader.read_exact(&mut npcs_data)?;
            let npcs = npcs_data
                .chunks_exact(349)
                .map(NpcRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.3. Separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            // ── 2.4. Extra objects ──
            let extras_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut extras_data = vec![0u8; extras_count * 200];
            reader.read_exact(&mut extras_data)?;
            let extra_objects = extras_data
                .chunks_exact(200)
                .map(ExtraObjectRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.5. Extra-object trailer ──
            let tail_size = reader.read_u32::<LittleEndian>()?;
            let trailer_record_count = reader.read_u16::<LittleEndian>()? as usize;
            let mut trailer_records_data = vec![0u8; trailer_record_count * 24];
            reader.read_exact(&mut trailer_records_data)?;
            let records = trailer_records_data
                .chunks_exact(24)
                .map(ExtraObjectTrailerRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;
            let automatic_placement_active = reader.read_u8()?;
            let automatic_placement_value = reader.read_u16::<LittleEndian>()?;
            let automatic_placement_global_item_index = reader.read_u16::<LittleEndian>()?;
            let extra_objects_trailer = MapExtraObjectsTrailer {
                tail_size,
                records,
                automatic_placement_active,
                automatic_placement_value,
                automatic_placement_global_item_index,
            };

            // ── 2.6–2.10. Ground items (5 types) ──
            let draw_items_weapon =
                Self::read_item_section(reader, 296, DrawItemWeaponItem::parse)?;
            let draw_items_heal = Self::read_item_section(reader, 264, DrawItemHealItem::parse)?;
            let draw_items_edit = Self::read_item_section(reader, 280, DrawItemEditItem::parse)?;
            let draw_items_misc = Self::read_item_section(reader, 268, DrawItemMiscItem::parse)?;
            let draw_items_event = Self::read_item_section(reader, 252, DrawItemEventItem::parse)?;

            let expected_tail_size = 17usize
                + extra_objects_trailer.records.len() * 24
                + draw_items_weapon.len() * 296
                + draw_items_heal.len() * 264
                + draw_items_edit.len() * 280
                + draw_items_misc.len() * 268
                + draw_items_event.len() * 252;
            if extra_objects_trailer.tail_size as usize != expected_tail_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "map extra-object trailer size is {}, expected {expected_tail_size}",
                        extra_objects_trailer.tail_size
                    ),
                ));
            }

            // ── 2.11. End-of-map separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            maps.push(MapSectionData {
                map_id,
                monsters,
                npcs,
                extra_objects,
                extra_objects_trailer,
                draw_items_weapon,
                draw_items_heal,
                draw_items_edit,
                draw_items_misc,
                draw_items_event,
            });
        }

        Ok(maps)
    }

    /// Parse the save-world header and player runtime-state snapshot.
    ///
    /// Layout: `[map-section terminator: u32][8 × 4-byte header values]
    /// [visited-map count][visited map IDs][player runtime state: 10,148 bytes]`.
    fn parse_post_maps_data<R: Read>(
        reader: &mut R,
        num_visited_maps: u32,
    ) -> std::io::Result<PostMapsData> {
        let map_section_terminator = reader.read_u32::<LittleEndian>()?;
        let game_version = reader.read_f32::<LittleEndian>()?;
        let unknown_header_value_1 = reader.read_u32::<LittleEndian>()?;
        let all_map_ini_id = reader.read_u32::<LittleEndian>()?;
        let ref_map_ini_id = reader.read_u32::<LittleEndian>()?;
        let monster_block_size = reader.read_u32::<LittleEndian>()?;
        let npc_block_size = reader.read_u32::<LittleEndian>()?;
        let unknown_header_value_2 = reader.read_u32::<LittleEndian>()?;
        let extra_object_block_size = reader.read_u32::<LittleEndian>()?;

        let number_of_visited_maps = reader.read_u32::<LittleEndian>()?;
        if number_of_visited_maps != num_visited_maps {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "post-maps visited-map count is {number_of_visited_maps}, expected {num_visited_maps}"
                ),
            ));
        }

        let mut map_ids = vec![0u32; number_of_visited_maps as usize];
        for map_id in &mut map_ids {
            *map_id = reader.read_u32::<LittleEndian>()?;
        }

        let map_viewport_state = MapViewportState::read_from(reader)?;

        Ok(PostMapsData {
            map_section_terminator,
            game_version,
            unknown_header_value_1,
            all_map_ini_id,
            ref_map_ini_id,
            monster_block_size,
            npc_block_size,
            unknown_header_value_2,
            extra_object_block_size,
            number_of_visited_maps,
            map_ids,
            map_viewport_state,
        })
    }

    /// Parse the 4 character sprite paths (4 × 60-byte fixed buffers).
    ///
    /// Each path is a null-terminated WINDOWS-1250 string, e.g.
    /// `"inter\\m_bald.spr"` or `"CharacterInGame\\m_warrior.spr"`.
    fn parse_sprite_paths<R: Read>(reader: &mut R) -> std::io::Result<Vec<String>> {
        let mut paths = Vec::with_capacity(4);
        for _ in 0..4 {
            let mut buf = [0u8; 60];
            reader.read_exact(&mut buf)?;
            paths.push(
                read_null_terminated_windows_1250(&buf)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            );
        }
        Ok(paths)
    }

    /// Parse belt data, character stats, and trailing unknown bytes.
    ///
    /// Layout:
    ///   `[unknown_before_stats_a: 8B][position_x: i16][position_y: i16]
    ///    [character_stats_header: 28B][strength u16][agility u16][wisdom u16][constitution u16]
    ///    [morale u16][hp_cur u16][hp_max u16][mp_cur u16][mp_max u16]
    ///    [xp u32][level u16][gold u32][offense u16][defense u16]
    ///    [dodge u8][hit u8][magic_power u16][attack_mod u8]
    ///    [thievery u8][lockpick u8][haggle u8][perception u8][traps u8]
    ///    [sword_lv u8][sword_kills u16][axe_lv u8][axe_kills u16]
    ///    [archery_lv u8][archery_kills u16][polearm_lv u8][polearm_kills u16]
    ///    [magic_lv u8][magic_kills u16][holy_lv u8][holy_kills u16]
    ///    [dark_lv u8][dark_kills u16][unknown: 9B]`
    #[allow(clippy::type_complexity)]
    fn parse_character_stats<R: Read>(
        reader: &mut R,
    ) -> std::io::Result<(
        Vec<u8>,
        i16,
        i16,
        CharacterStatsHeader,
        CharacterStats,
        Vec<u8>,
    )> {
        // ── Leading data (8 bytes, purpose unknown) ──
        let mut unknown_before_stats = vec![0u8; 8];
        reader.read_exact(&mut unknown_before_stats)?;

        let character_position_x = reader.read_i16::<LittleEndian>()?;
        let character_position_y = reader.read_i16::<LittleEndian>()?;

        // ── Character stats header (28 bytes) ──
        let character_stats_header = CharacterStatsHeader {
            unknown_a: reader.read_u8()?,
            unknown_b: reader.read_u32::<LittleEndian>()?,
            selected_spell_id: reader.read_u32::<LittleEndian>()?,
            unknown_block: {
                let mut bytes = [0u8; 19];
                reader.read_exact(&mut bytes)?;
                bytes
            },
        };

        // ── Structured stats block ──
        let character_stats = CharacterStats {
            strength: reader.read_u16::<LittleEndian>()?,
            agility: reader.read_u16::<LittleEndian>()?,
            wisdom: reader.read_u16::<LittleEndian>()?,
            constitution: reader.read_u16::<LittleEndian>()?,
            morale: reader.read_u16::<LittleEndian>()?,
            hp_current: reader.read_u16::<LittleEndian>()?,
            hp_maximum: reader.read_u16::<LittleEndian>()?,
            mp_current: reader.read_u16::<LittleEndian>()?,
            mp_maximum: reader.read_u16::<LittleEndian>()?,
            experience: reader.read_u32::<LittleEndian>()?,
            level: reader.read_u16::<LittleEndian>()?,
            gold: reader.read_u32::<LittleEndian>()?,
            offense: reader.read_u16::<LittleEndian>()?,
            defense: reader.read_u16::<LittleEndian>()?,
            dodge_rate: reader.read_u8()?,
            hit_rate: reader.read_u8()?,
            magic_power: reader.read_u16::<LittleEndian>()?,
            attack_modifier: reader.read_u8()?,
            pickpocketing: reader.read_u8()?,
            lockpicking: reader.read_u8()?,
            haggling: reader.read_u8()?,
            perception: reader.read_u8()?,
            traps: reader.read_u8()?,
            swords_level: reader.read_u8()?,
            swords_kills: reader.read_u16::<LittleEndian>()?,
            axes_level: reader.read_u8()?,
            axes_kills: reader.read_u16::<LittleEndian>()?,
            archery_level: reader.read_u8()?,
            archery_kills: reader.read_u16::<LittleEndian>()?,
            polearm_level: reader.read_u8()?,
            polearm_kills: reader.read_u16::<LittleEndian>()?,
            magic_level: reader.read_u8()?,
            magic_kills: reader.read_u16::<LittleEndian>()?,
            holy_magic_level: reader.read_u8()?,
            holy_magic_kills: reader.read_u16::<LittleEndian>()?,
            dark_magic_level: reader.read_u8()?,
            dark_magic_kills: reader.read_u16::<LittleEndian>()?,
        };

        // ── Trailing unknown bytes ──
        let mut unknown_after_stats = vec![0u8; 9];
        reader.read_exact(&mut unknown_after_stats)?;

        Ok((
            unknown_before_stats,
            character_position_x,
            character_position_y,
            character_stats_header,
            character_stats,
            unknown_after_stats,
        ))
    }

    /// Parse the inventory section (5 count-prefixed item categories).
    ///
    /// Record sizes: Event=244, Misc=264, Edit=272, Weapon=292, Heal=256.
    fn parse_inventory_section<R: Read>(reader: &mut R) -> std::io::Result<InventoryData> {
        Ok(InventoryData {
            event_items: Self::read_item_section(reader, 244, InventoryEventItem::parse)?,
            misc_items: Self::read_item_section(reader, 264, InventoryMiscItem::parse)?,
            edit_items: Self::read_item_section(reader, 272, InventoryEditItem::parse)?,
            weapon_items: Self::read_item_section(reader, 292, InventoryWeaponItem::parse)?,
            heal_items: Self::read_item_section(reader, 256, InventoryHealItem::parse)?,
        })
    }

    /// Parse the journal section (42-byte header + 3 × 100 × 37-byte entries).
    fn parse_journal_section<R: Read>(reader: &mut R) -> std::io::Result<JournalData> {
        const HEADER_SIZE: usize = 42;
        const ENTRY_SIZE: usize = 37;
        const ENTRIES_PER_SECTION: usize = 100;
        const SECTION_SIZE: usize = ENTRY_SIZE * ENTRIES_PER_SECTION; // 3700

        let mut header_data = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_data)?;
        let header = JournalHeader::parse(&header_data)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let main = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let side = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let trade = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        Ok(JournalData {
            header,
            main,
            side,
            trade,
        })
    }

    /// Parse the events section (2251 × 284-byte event records).
    fn parse_events_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<EventRecord>> {
        const EVENT_COUNT: usize = 2251;
        const EVENT_SIZE: usize = 284;

        let mut events: Vec<EventRecord> = Vec::with_capacity(EVENT_COUNT);
        for _ in 0..EVENT_COUNT {
            let mut buf = [0u8; EVENT_SIZE];
            reader.read_exact(&mut buf)?;
            events.push(EventRecord::parse(&buf)?);
        }
        Ok(events)
    }

    /// Parse character identity (name, class, equipment, spells, party).
    ///
    /// Layout:
    ///   `[unknown_96B][name: 11B][class_id: u16][class_name: 11B]
    ///    [header: 11B][equipment: 108B][belt: 96B][inventory: 3780B][spells: 41B]
    ///    [party_count: u32][party_members]`
    fn parse_character_identity<R: Read + Seek>(
        reader: &mut R,
    ) -> std::io::Result<CharacterIdentity> {
        // ── 7.1. Unknown block (96 bytes before name) ──
        let mut unknown_block = vec![0u8; 96];
        reader.read_exact(&mut unknown_block)?;

        // ── 7.2. Player name (11-byte WINDOWS-1250 null-terminated) ──
        let mut name_raw = vec![0u8; 11];
        reader.read_exact(&mut name_raw)?;
        let player_name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // ── 7.3. Player class ──
        let player_class_id = reader.read_u16::<LittleEndian>()?;
        let mut class_raw = vec![0u8; 11];
        reader.read_exact(&mut class_raw)?;
        let player_class_name = read_null_terminated_windows_1250(&class_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // ── 7.4. Character data blocks ──
        // TODO: Add writer method for these blocks

        // Header: u32 + u16 + u16 + u8 + u8 + u8 = 11 bytes
        let mut header_buf = [0u8; 11];
        reader.read_exact(&mut header_buf)?;
        let character_data_header = CharacterDataHeader::parse(&header_buf)?;

        // Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
        let mut equipment_raw = vec![0u8; 12 * 9];
        reader.read_exact(&mut equipment_raw)?;
        let equipped_equipment: Vec<EquipmentSlot> = equipment_raw
            .chunks_exact(9)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                EquipmentSlot {
                    panel_slot_marker: c.read_u8().unwrap(),
                    weapon_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    weapon_inventory_instance_id: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Belt item placements: 6 cells × 16 bytes = 96 bytes.
        let mut belt_raw = vec![0u8; 6 * 16];
        reader.read_exact(&mut belt_raw)?;
        let belt_potions: Vec<BeltPotionSlot> = belt_raw
            .chunks_exact(16)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                BeltPotionSlot {
                    item_category: c.read_i32::<LittleEndian>().unwrap(),
                    item_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    icon_x: c.read_i32::<LittleEndian>().unwrap(),
                    icon_y: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes.
        let mut inventory_raw = vec![0u8; 189 * 20];
        reader.read_exact(&mut inventory_raw)?;
        let inventory_placement: Vec<InventoryPlacementEntry> = inventory_raw
            .chunks_exact(20)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                InventoryPlacementEntry {
                    item_category: c.read_i32::<LittleEndian>().unwrap(),
                    item_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    icon_x: c.read_i32::<LittleEndian>().unwrap(),
                    icon_y: c.read_i32::<LittleEndian>().unwrap(),
                    item_instance_index: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Learned spells: 41 bytes (one flag per spell)
        let mut spells_buf = vec![0u8; 41];
        reader.read_exact(&mut spells_buf)?;
        let learned_spells = LearnedSpells { spells: spells_buf };

        // ── 7.5. Party members ──
        let party_members_count = reader.read_u32::<LittleEndian>()?;
        let mut party_members = Vec::with_capacity(party_members_count as usize);
        for _ in 0..party_members_count {
            party_members.push(PartyMember::read_from(reader)?);
        }

        Ok(CharacterIdentity {
            unknown_block,
            player_name,
            player_class_id,
            player_class_name,
            character_data_header,
            equipped_equipment,
            belt_potions,
            inventory_placement,
            learned_spells,
            party_members_count,
            party_members,
        })
    }

    /// Parse the unknown section between events and journal.
    ///
    /// Layout: `[block_a: 12B][count: u32][count × 24B records][block_b: 56B]`
    fn parse_post_events_data<R: Read>(reader: &mut R) -> std::io::Result<PostEventsData> {
        let mut block_a = vec![0u8; 12];
        reader.read_exact(&mut block_a)?;

        let count = reader.read_u32::<LittleEndian>()? as usize;
        let mut records = vec![0u8; count * 24];
        reader.read_exact(&mut records)?;

        let mut block_b = vec![0u8; 56];
        reader.read_exact(&mut block_b)?;

        Ok(PostEventsData {
            block_a,
            records,
            block_b,
        })
    }

    /// Parse journal entries from raw binary data
    fn parse_journal_entries(data: &[u8], count: usize) -> std::io::Result<Vec<JournalEntry>> {
        let expected_len = count * 37;
        if data.len() < expected_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Journal data too short",
            ));
        }

        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * 37;
            let entry = JournalEntry::parse(&data[offset..offset + 37])?;
            entries.push(entry);
        }

        Ok(entries)
    }

    // ── Write helpers ─────────────────────────────────────────────────────────

    /// Write the maps section to a writer (used internally to pre-compute size).
    fn write_maps_section<W: Write>(
        maps: &[MapSectionData],
        writer: &mut W,
    ) -> std::io::Result<()> {
        for map in maps {
            writer.write_u32::<LittleEndian>(map.map_id)?;

            // Monsters: u32 count + 329-byte records
            writer.write_u32::<LittleEndian>(map.monsters.len() as u32)?;
            for m in &map.monsters {
                m.write(writer)?;
            }

            // NPCs: u32 count + 349-byte records
            writer.write_u32::<LittleEndian>(map.npcs.len() as u32)?;
            for n in &map.npcs {
                n.write(writer)?;
            }

            // Separator (always 0)
            writer.write_u32::<LittleEndian>(0)?;

            // Extra objects: u32 count + 200-byte records
            writer.write_u32::<LittleEndian>(map.extra_objects.len() as u32)?;
            for e in &map.extra_objects {
                e.write(writer)?;
            }

            // Extra-object trailer: size, count, records, then controls.
            let record_count =
                u16::try_from(map.extra_objects_trailer.records.len()).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "map extra-object trailer has more than u16::MAX records",
                    )
                })?;
            let expected_tail_size = 17usize
                + map.extra_objects_trailer.records.len() * 24
                + map.draw_items_weapon.len() * 296
                + map.draw_items_heal.len() * 264
                + map.draw_items_edit.len() * 280
                + map.draw_items_misc.len() * 268
                + map.draw_items_event.len() * 252;
            if map.extra_objects_trailer.tail_size as usize != expected_tail_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "map extra-object trailer size is {}, expected {expected_tail_size}",
                        map.extra_objects_trailer.tail_size
                    ),
                ));
            }
            writer.write_u32::<LittleEndian>(map.extra_objects_trailer.tail_size)?;
            writer.write_u16::<LittleEndian>(record_count)?;
            for record in &map.extra_objects_trailer.records {
                record.write(writer)?;
            }
            writer.write_u8(map.extra_objects_trailer.automatic_placement_active)?;
            writer
                .write_u16::<LittleEndian>(map.extra_objects_trailer.automatic_placement_value)?;
            writer.write_u16::<LittleEndian>(
                map.extra_objects_trailer
                    .automatic_placement_global_item_index,
            )?;

            // Ground items (5 types, each u16 count + fixed-size records)
            writer.write_u16::<LittleEndian>(map.draw_items_weapon.len() as u16)?;
            for d in &map.draw_items_weapon {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_heal.len() as u16)?;
            for d in &map.draw_items_heal {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_edit.len() as u16)?;
            for d in &map.draw_items_edit {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_misc.len() as u16)?;
            for d in &map.draw_items_misc {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_event.len() as u16)?;
            for d in &map.draw_items_event {
                d.write(writer)?;
            }

            // End-of-map separator (always 0)
            writer.write_u32::<LittleEndian>(0)?;
        }
        Ok(())
    }

    /// Write post-maps data block.
    fn write_post_maps_data<W: Write>(data: &PostMapsData, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(data.map_section_terminator)?;
        writer.write_f32::<LittleEndian>(data.game_version)?;
        writer.write_u32::<LittleEndian>(data.unknown_header_value_1)?;
        writer.write_u32::<LittleEndian>(data.all_map_ini_id)?;
        writer.write_u32::<LittleEndian>(data.ref_map_ini_id)?;
        writer.write_u32::<LittleEndian>(data.monster_block_size)?;
        writer.write_u32::<LittleEndian>(data.npc_block_size)?;
        writer.write_u32::<LittleEndian>(data.unknown_header_value_2)?;
        writer.write_u32::<LittleEndian>(data.extra_object_block_size)?;
        let map_id_count = u32::try_from(data.map_ids.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "post-maps has more than u32::MAX map IDs",
            )
        })?;
        if data.number_of_visited_maps != map_id_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "post-maps visited-map count is {}, but {} map IDs were provided",
                    data.number_of_visited_maps,
                    data.map_ids.len()
                ),
            ));
        }
        writer.write_u32::<LittleEndian>(data.number_of_visited_maps)?;
        for id in &data.map_ids {
            writer.write_u32::<LittleEndian>(*id)?;
        }
        data.map_viewport_state.write_to(writer)?;
        Ok(())
    }

    /// Write sprite paths (always 4 × 60-byte fixed buffers).
    fn write_sprite_paths<W: Write>(paths: &[String], writer: &mut W) -> std::io::Result<()> {
        for i in 0..4 {
            let s = paths.get(i).map(|s| s.as_str()).unwrap_or("");
            let mut buf = [0u8; 60];
            let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(s);
            let len = std::cmp::min(cow.len(), 60);
            buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&buf)?;
        }
        Ok(())
    }

    /// Write position data, character stats, and trailing unknown bytes.
    fn write_character_stats<W: Write>(
        unknown_before_a: &[u8],
        character_position_x: i16,
        character_position_y: i16,
        character_stats_header: &CharacterStatsHeader,
        stats: &CharacterStats,
        unknown_after: &[u8],
        writer: &mut W,
    ) -> std::io::Result<()> {
        writer.write_all(unknown_before_a)?;
        writer.write_i16::<LittleEndian>(character_position_x)?;
        writer.write_i16::<LittleEndian>(character_position_y)?;
        writer.write_u8(character_stats_header.unknown_a)?;
        writer.write_u32::<LittleEndian>(character_stats_header.unknown_b)?;
        writer.write_u32::<LittleEndian>(character_stats_header.selected_spell_id)?;
        writer.write_all(&character_stats_header.unknown_block)?;
        writer.write_u16::<LittleEndian>(stats.strength)?;
        writer.write_u16::<LittleEndian>(stats.agility)?;
        writer.write_u16::<LittleEndian>(stats.wisdom)?;
        writer.write_u16::<LittleEndian>(stats.constitution)?;
        writer.write_u16::<LittleEndian>(stats.morale)?;
        writer.write_u16::<LittleEndian>(stats.hp_current)?;
        writer.write_u16::<LittleEndian>(stats.hp_maximum)?;
        writer.write_u16::<LittleEndian>(stats.mp_current)?;
        writer.write_u16::<LittleEndian>(stats.mp_maximum)?;
        writer.write_u32::<LittleEndian>(stats.experience)?;
        writer.write_u16::<LittleEndian>(stats.level)?;
        writer.write_u32::<LittleEndian>(stats.gold)?;
        writer.write_u16::<LittleEndian>(stats.offense)?;
        writer.write_u16::<LittleEndian>(stats.defense)?;
        writer.write_u8(stats.dodge_rate)?;
        writer.write_u8(stats.hit_rate)?;
        writer.write_u16::<LittleEndian>(stats.magic_power)?;
        writer.write_u8(stats.attack_modifier)?;
        writer.write_u8(stats.pickpocketing)?;
        writer.write_u8(stats.lockpicking)?;
        writer.write_u8(stats.haggling)?;
        writer.write_u8(stats.perception)?;
        writer.write_u8(stats.traps)?;
        writer.write_u8(stats.swords_level)?;
        writer.write_u16::<LittleEndian>(stats.swords_kills)?;
        writer.write_u8(stats.axes_level)?;
        writer.write_u16::<LittleEndian>(stats.axes_kills)?;
        writer.write_u8(stats.archery_level)?;
        writer.write_u16::<LittleEndian>(stats.archery_kills)?;
        writer.write_u8(stats.polearm_level)?;
        writer.write_u16::<LittleEndian>(stats.polearm_kills)?;
        writer.write_u8(stats.magic_level)?;
        writer.write_u16::<LittleEndian>(stats.magic_kills)?;
        writer.write_u8(stats.holy_magic_level)?;
        writer.write_u16::<LittleEndian>(stats.holy_magic_kills)?;
        writer.write_u8(stats.dark_magic_level)?;
        writer.write_u16::<LittleEndian>(stats.dark_magic_kills)?;
        writer.write_all(unknown_after)?;
        Ok(())
    }

    /// Write inventory (5 categories, each u16 count + fixed-size records).
    fn write_inventory<W: Write>(inv: &InventoryData, writer: &mut W) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(inv.event_items.len() as u16)?;
        for item in &inv.event_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.misc_items.len() as u16)?;
        for item in &inv.misc_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.edit_items.len() as u16)?;
        for item in &inv.edit_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.weapon_items.len() as u16)?;
        for item in &inv.weapon_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.heal_items.len() as u16)?;
        for item in &inv.heal_items {
            item.write(writer)?;
        }
        Ok(())
    }

    /// Write character identity (96B unknown + 11B name + u16 class + 11B class name + character data blocks + party).
    fn write_character_identity<W: Write>(
        identity: &CharacterIdentity,
        writer: &mut W,
    ) -> std::io::Result<()> {
        writer.write_all(&identity.unknown_block)?;

        // Player name: 11-byte WINDOWS-1250 fixed buffer
        let mut name_buf = [0u8; 11];
        let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(&identity.player_name);
        let len = std::cmp::min(cow.len(), 11);
        name_buf[..len].copy_from_slice(&cow[..len]);
        writer.write_all(&name_buf)?;

        writer.write_u16::<LittleEndian>(identity.player_class_id)?;

        // Class name: 11-byte WINDOWS-1250 fixed buffer
        let mut class_buf = [0u8; 11];
        let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(&identity.player_class_name);
        let len = std::cmp::min(cow.len(), 11);
        class_buf[..len].copy_from_slice(&cow[..len]);
        writer.write_all(&class_buf)?;

        // ── Character data blocks ──

        // Header: u32 + u16 + u16 + u8 + u8 + u8 = 11 bytes
        identity.character_data_header.write(writer)?;

        // Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
        for slot in &identity.equipped_equipment {
            writer.write_u8(slot.panel_slot_marker)?;
            writer.write_i32::<LittleEndian>(slot.weapon_catalog_index)?;
            writer.write_i32::<LittleEndian>(slot.weapon_inventory_instance_id)?;
        }

        // Belt item placements: 6 cells × 16 bytes = 96 bytes.
        for slot in &identity.belt_potions {
            writer.write_i32::<LittleEndian>(slot.item_category)?;
            writer.write_i32::<LittleEndian>(slot.item_catalog_index)?;
            writer.write_i32::<LittleEndian>(slot.icon_x)?;
            writer.write_i32::<LittleEndian>(slot.icon_y)?;
        }

        // Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes.
        for entry in &identity.inventory_placement {
            writer.write_i32::<LittleEndian>(entry.item_category)?;
            writer.write_i32::<LittleEndian>(entry.item_catalog_index)?;
            writer.write_i32::<LittleEndian>(entry.icon_x)?;
            writer.write_i32::<LittleEndian>(entry.icon_y)?;
            writer.write_i32::<LittleEndian>(entry.item_instance_index)?;
        }

        // Learned spells: 41 bytes
        writer.write_all(&identity.learned_spells.spells)?;

        // ── Party members ──
        writer.write_u32::<LittleEndian>(identity.party_members_count)?;
        for member in &identity.party_members {
            member.write(writer)?;
        }
        Ok(())
    }

    /// Write event scripts in order.
    fn write_events<W: Write>(events: &[EventRecord], writer: &mut W) -> std::io::Result<()> {
        for event in events {
            event.write(writer)?;
        }
        Ok(())
    }

    /// Write post-events unknown data block.
    fn write_post_events<W: Write>(data: &PostEventsData, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&data.block_a)?;
        let count = data.records.len() / 24;
        writer.write_u32::<LittleEndian>(count as u32)?;
        writer.write_all(&data.records)?;
        writer.write_all(&data.block_b)?;
        Ok(())
    }

    /// Write journal (3 sections × entries in order).
    fn write_journal<W: Write>(journal: &JournalData, writer: &mut W) -> std::io::Result<()> {
        journal.header.write(writer)?;
        for entry in &journal.main {
            entry.write(writer)?;
        }
        for entry in &journal.side {
            entry.write(writer)?;
        }
        for entry in &journal.trade {
            entry.write(writer)?;
        }
        Ok(())
    }
}

impl Extractor for SaveFile {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let save = SaveFile::parse(&data)?;
        Ok(vec![save])
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }
        let save = &records[0];

        // Pre-compute maps section to determine jump_addr_after_maps
        let mut maps_buf = Vec::new();
        Self::write_maps_section(&save.maps, &mut maps_buf)?;
        let jump_addr = 8u32 + maps_buf.len() as u32;

        // 1. Header: jump address after all maps data
        writer.write_u32::<LittleEndian>(jump_addr)?;

        // 2. Map count + maps data
        writer.write_u32::<LittleEndian>(save.maps.len() as u32)?;
        writer.write_all(&maps_buf)?;

        // 3. Post-maps data
        Self::write_post_maps_data(&save.post_maps, writer)?;

        // 4. Sprite paths (always 4 × 60-byte fixed buffers)
        Self::write_sprite_paths(&save.sprite_paths, writer)?;

        // 5. Belt data + character stats + trailing bytes
        Self::write_character_stats(
            &save.unknown_before_stats,
            save.character_position_x,
            save.character_position_y,
            &save.character_stats_header,
            &save.character_stats,
            &save.unknown_after_stats,
            writer,
        )?;

        // 6. Inventory (5 categories)
        Self::write_inventory(&save.inventory, writer)?;

        // 7. Character identity
        Self::write_character_identity(&save.character_identity, writer)?;

        // 8. Events
        Self::write_events(&save.events, writer)?;

        // 9. Post-events data
        Self::write_post_events(&save.post_events, writer)?;

        // 10. Journal (3 sections)
        Self::write_journal(&save.journal, writer)?;

        Ok(())
    }
}
