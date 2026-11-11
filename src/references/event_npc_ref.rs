use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use crate::references::enums::{
    BooleanFlag, NpcInteractionMode, NpcLookingDirection, NpcMovementMode, NpcRoleResult,
};
use crate::references::extractor::Extractor;
use dispel_macros::{Localizable, TextRecordPatcher};
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// Number of comma-separated columns in one `Eventnpc.ref` row.
///
/// The game's loader (`eventnpc_ref_load` in `Dispel_145_wm.exe`) reads the
/// file with `strtok` over a comma delimiter and consumes exactly 40 tokens
/// per row: one row id, one NPC archetype id, two strings, three flags, four
/// waypoint blocks of six values each, the activation rectangle, and the
/// interaction block.
pub const COLUMN_COUNT: usize = 40;

/// EventNpc.ref - Event NPC Definitions
///
/// Stores the full placement record for NPCs that exist only while a
/// scripted event runs. The row layout mirrors the 672-byte binary
/// `NpcInGame/Npccat*.ref` record, because the game's `eventnpc_ref_load`
/// feeds the same runtime NPC structure that the binary loader fills in.
/// See `docs/files/NpcInGame/Eventnpc.ref.md` for the column table.
///
/// Reads file: `NpcInGame/Eventnpc.ref`
///
/// # ASCII Structure
///
/// ```text
/// ; Comment line
/// id,npc_ini_id,name,role,role_result,show_on_event,movement_mode,
/// wp1_filled,wp1_x,wp1_y,wp1_wait,wp1_facing,wp1_reserved, ... ,
/// wp4_reserved,rect_x1,rect_y1,rect_x2,rect_y2,
/// interaction_mode,interaction_result,interaction_range,dialog_id,face_sprite
/// ```
///
/// A well-formed row has 40 columns. The shipped game data contains one row
/// whose comma between the `npc_ini_id` and the `name` column is missing; the
/// parser repairs that row (see [`split_glued_id_and_name`]).
///
/// # Field Definitions
///
/// - `id`: Row index the event opcodes use to address this NPC. The game
///   itself ignores the value and selects rows by position.
/// - `npc_ini_id`: NPC visual-archetype ID linked from `Npc.ini`.
/// - `name`: Display name shown by the game.
/// - `role_description`: Role or descriptive text stored with the NPC.
/// - `role_result`: Role behavior selected when the NPC is interacted with.
/// - `show_on_event`: Event ID condition required to spawn the NPC.
/// - `movement_mode`: Static, waypoint patrol, or random walk in the
///   activation rectangle.
/// - `waypointN_*`: Four patrol waypoints, each stored as
///   `filled, x, y, wait_time, facing_direction, reserved`.
/// - `activation_rect_*`: Rectangle the NPC walks inside when
///   `movement_mode` is `RandomInActivationRect`.
/// - `interaction_mode`: Selects the special interaction-result behavior.
/// - `interaction_result`: Packed interaction result. The low 16 bits hold
///   the item id, the high 16 bits the parameter.
/// - `interaction_range_offset`: Value added to one by the game for its
///   interaction-distance comparison.
/// - `dialog_id`: Dialogue triggered when the NPC is clicked.
/// - `dialogue_face_sprite_id`: Portrait sprite shown in dialogue windows.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Localizable, TextRecordPatcher)]
#[patcher(filename = "Eventnpc.ref")]
pub struct EventNpcRef {
    /// Row index used by the event opcodes to address this NPC.
    #[extractor(field = 0)]
    pub id: i32,
    /// NPC visual-archetype ID linked from `Npc.ini`.
    #[extractor(field = 1)]
    pub npc_ini_id: i32,
    /// Display name shown by the game.
    #[extractor(field = 2)]
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 260)]
    pub name: String,
    /// Role or descriptive text stored with the NPC.
    #[extractor(field = 3)]
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 260)]
    pub role_description: String,
    /// Role behavior selected when the NPC is interacted with.
    #[extractor(field = 4, enum_from_i32(type = "NpcRoleResult"))]
    pub role_result: NpcRoleResult,
    /// Event ID condition required to spawn the NPC.
    #[extractor(field = 5)]
    pub show_on_event: i32,
    /// Movement pattern: static, waypoint patrol, or random movement in the
    /// activation rectangle.
    #[extractor(field = 6, enum_from_i32(type = "NpcMovementMode"))]
    pub movement_mode: NpcMovementMode,
    /// Waypoint 1 definition flag.
    #[extractor(field = 7, enum_from_i32(type = "BooleanFlag"))]
    pub waypoint1_filled: BooleanFlag,
    /// Waypoint 1 X target.
    #[extractor(field = 8)]
    pub waypoint1_x: i32,
    /// Waypoint 1 Y target.
    #[extractor(field = 9)]
    pub waypoint1_y: i32,
    /// Wait time before the NPC moves from waypoint 1.
    #[extractor(field = 10)]
    pub waypoint1_wait_time: i32,
    /// Facing direction at waypoint 1.
    #[extractor(field = 11, enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint1_facing_direction: NpcLookingDirection,
    /// Reserved value for waypoint 1. Observed as zero.
    #[extractor(field = 12)]
    pub waypoint1_reserved: i32,
    /// Waypoint 2 definition flag.
    #[extractor(field = 13, enum_from_i32(type = "BooleanFlag"))]
    pub waypoint2_filled: BooleanFlag,
    /// Waypoint 2 X target.
    #[extractor(field = 14)]
    pub waypoint2_x: i32,
    /// Waypoint 2 Y target.
    #[extractor(field = 15)]
    pub waypoint2_y: i32,
    /// Wait time before the NPC moves from waypoint 2.
    #[extractor(field = 16)]
    pub waypoint2_wait_time: i32,
    /// Facing direction at waypoint 2.
    #[extractor(field = 17, enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint2_facing_direction: NpcLookingDirection,
    /// Reserved value for waypoint 2. Observed as zero.
    #[extractor(field = 18)]
    pub waypoint2_reserved: i32,
    /// Waypoint 3 definition flag.
    #[extractor(field = 19, enum_from_i32(type = "BooleanFlag"))]
    pub waypoint3_filled: BooleanFlag,
    /// Waypoint 3 X target.
    #[extractor(field = 20)]
    pub waypoint3_x: i32,
    /// Waypoint 3 Y target.
    #[extractor(field = 21)]
    pub waypoint3_y: i32,
    /// Wait time before the NPC moves from waypoint 3.
    #[extractor(field = 22)]
    pub waypoint3_wait_time: i32,
    /// Facing direction at waypoint 3.
    #[extractor(field = 23, enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint3_facing_direction: NpcLookingDirection,
    /// Reserved value for waypoint 3. Observed as zero.
    #[extractor(field = 24)]
    pub waypoint3_reserved: i32,
    /// Waypoint 4 definition flag.
    #[extractor(field = 25, enum_from_i32(type = "BooleanFlag"))]
    pub waypoint4_filled: BooleanFlag,
    /// Waypoint 4 X target.
    #[extractor(field = 26)]
    pub waypoint4_x: i32,
    /// Waypoint 4 Y target.
    #[extractor(field = 27)]
    pub waypoint4_y: i32,
    /// Wait time before the NPC moves from waypoint 4.
    #[extractor(field = 28)]
    pub waypoint4_wait_time: i32,
    /// Facing direction at waypoint 4.
    #[extractor(field = 29, enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint4_facing_direction: NpcLookingDirection,
    /// Reserved value for waypoint 4. Observed as zero.
    #[extractor(field = 30)]
    pub waypoint4_reserved: i32,
    /// Activation rectangle X1 coordinate.
    #[extractor(field = 31)]
    pub activation_rect_x1: i32,
    /// Activation rectangle Y1 coordinate.
    #[extractor(field = 32)]
    pub activation_rect_y1: i32,
    /// Activation rectangle X2 coordinate.
    #[extractor(field = 33)]
    pub activation_rect_x2: i32,
    /// Activation rectangle Y2 coordinate.
    #[extractor(field = 34)]
    pub activation_rect_y2: i32,
    /// Selects the special interaction-result behavior.
    #[extractor(field = 35, enum_from_i32(type = "NpcInteractionMode"))]
    pub interaction_mode: NpcInteractionMode,
    /// Packed interaction result. Low 16 bits: item id. High 16 bits: parameter.
    #[extractor(field = 36)]
    pub interaction_result: i32,
    /// Value added to one by the game for its interaction-distance comparison.
    #[extractor(field = 37)]
    pub interaction_range_offset: i32,
    /// Dialogue triggered when the NPC is clicked.
    #[extractor(field = 38)]
    pub dialog_id: i32,
    /// Portrait sprite shown in dialogue windows.
    #[extractor(field = 39)]
    pub dialogue_face_sprite_id: i32,
}

impl EventNpcRef {
    /// Build a record from the columns of one row, in file order.
    fn from_columns(columns: &[&str]) -> Self {
        let int = |index: usize| -> i32 { columns[index].trim().parse::<i32>().unwrap_or(0) };
        let text = |index: usize| -> String { columns[index].to_string() };

        EventNpcRef {
            id: int(0),
            npc_ini_id: int(1),
            name: text(2),
            role_description: text(3),
            role_result: NpcRoleResult::from_i32(int(4)).unwrap_or_default(),
            show_on_event: int(5),
            movement_mode: NpcMovementMode::from_i32(int(6)).unwrap_or_default(),
            waypoint1_filled: BooleanFlag::from_i32(int(7)).unwrap_or_default(),
            waypoint1_x: int(8),
            waypoint1_y: int(9),
            waypoint1_wait_time: int(10),
            waypoint1_facing_direction: NpcLookingDirection::from_i32(int(11)).unwrap_or_default(),
            waypoint1_reserved: int(12),
            waypoint2_filled: BooleanFlag::from_i32(int(13)).unwrap_or_default(),
            waypoint2_x: int(14),
            waypoint2_y: int(15),
            waypoint2_wait_time: int(16),
            waypoint2_facing_direction: NpcLookingDirection::from_i32(int(17)).unwrap_or_default(),
            waypoint2_reserved: int(18),
            waypoint3_filled: BooleanFlag::from_i32(int(19)).unwrap_or_default(),
            waypoint3_x: int(20),
            waypoint3_y: int(21),
            waypoint3_wait_time: int(22),
            waypoint3_facing_direction: NpcLookingDirection::from_i32(int(23)).unwrap_or_default(),
            waypoint3_reserved: int(24),
            waypoint4_filled: BooleanFlag::from_i32(int(25)).unwrap_or_default(),
            waypoint4_x: int(26),
            waypoint4_y: int(27),
            waypoint4_wait_time: int(28),
            waypoint4_facing_direction: NpcLookingDirection::from_i32(int(29)).unwrap_or_default(),
            waypoint4_reserved: int(30),
            activation_rect_x1: int(31),
            activation_rect_y1: int(32),
            activation_rect_x2: int(33),
            activation_rect_y2: int(34),
            interaction_mode: NpcInteractionMode::from_i32(int(35)).unwrap_or_default(),
            interaction_result: int(36),
            interaction_range_offset: int(37),
            dialog_id: int(38),
            dialogue_face_sprite_id: int(39),
        }
    }

    /// Render the record as the row columns, in file order.
    fn to_columns(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.npc_ini_id.to_string(),
            self.name.clone(),
            self.role_description.clone(),
            i32::from(self.role_result).to_string(),
            self.show_on_event.to_string(),
            i32::from(self.movement_mode).to_string(),
            i32::from(self.waypoint1_filled).to_string(),
            self.waypoint1_x.to_string(),
            self.waypoint1_y.to_string(),
            self.waypoint1_wait_time.to_string(),
            i32::from(self.waypoint1_facing_direction).to_string(),
            self.waypoint1_reserved.to_string(),
            i32::from(self.waypoint2_filled).to_string(),
            self.waypoint2_x.to_string(),
            self.waypoint2_y.to_string(),
            self.waypoint2_wait_time.to_string(),
            i32::from(self.waypoint2_facing_direction).to_string(),
            self.waypoint2_reserved.to_string(),
            i32::from(self.waypoint3_filled).to_string(),
            self.waypoint3_x.to_string(),
            self.waypoint3_y.to_string(),
            self.waypoint3_wait_time.to_string(),
            i32::from(self.waypoint3_facing_direction).to_string(),
            self.waypoint3_reserved.to_string(),
            i32::from(self.waypoint4_filled).to_string(),
            self.waypoint4_x.to_string(),
            self.waypoint4_y.to_string(),
            self.waypoint4_wait_time.to_string(),
            i32::from(self.waypoint4_facing_direction).to_string(),
            self.waypoint4_reserved.to_string(),
            self.activation_rect_x1.to_string(),
            self.activation_rect_y1.to_string(),
            self.activation_rect_x2.to_string(),
            self.activation_rect_y2.to_string(),
            i32::from(self.interaction_mode).to_string(),
            self.interaction_result.to_string(),
            self.interaction_range_offset.to_string(),
            self.dialog_id.to_string(),
            self.dialogue_face_sprite_id.to_string(),
        ]
    }
}

/// Split a column that carries both the `npc_ini_id` and the display name.
///
/// The shipped `Eventnpc.ref` holds one row where the comma between the
/// `npc_ini_id` and the `name` column is missing, so both values arrive glued
/// together (`"69Ronin Galuc"`). The game reads the leading digits as the id
/// and loses the name; the extractor splits the column so the record keeps
/// its intended values.
fn split_glued_id_and_name(column: &str) -> Option<(&str, &str)> {
    let digits_end = column
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .map(|(index, c)| index + c.len_utf8())
        .last()?;
    if digits_end == 0 || digits_end == column.len() {
        return None;
    }
    Some((&column[..digits_end], &column[digits_end..]))
}

impl Extractor for EventNpcRef {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);

        let mut records = Vec::new();
        for line in buf_reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }

            let mut columns: Vec<&str> = trimmed.split(',').collect();

            // Repair the shipped row whose `npc_ini_id` and `name` columns
            // arrive glued together by a missing comma.
            if columns.len() == COLUMN_COUNT - 1
                && let Some((id, name)) = split_glued_id_and_name(columns[1])
            {
                columns[1] = id;
                columns.insert(2, name);
            }

            if columns.len() < COLUMN_COUNT {
                eprintln!(
                    "Eventnpc.ref: row {} has {} columns, expected {COLUMN_COUNT}; \
                     padding the missing trailing columns",
                    records.len(),
                    columns.len()
                );
                columns.resize(COLUMN_COUNT, "");
            } else if columns.len() > COLUMN_COUNT {
                eprintln!(
                    "Eventnpc.ref: row {} has {} columns, expected {COLUMN_COUNT}; \
                     ignoring the extra columns",
                    records.len(),
                    columns.len()
                );
                columns.truncate(COLUMN_COUNT);
            }

            records.push(EventNpcRef::from_columns(&columns));
        }

        Ok(records)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let line = format!("{}\r\n", record.to_columns().join(","));
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_event_npc_ref(source_path: &Path) -> std::io::Result<Vec<EventNpcRef>> {
    EventNpcRef::read_file(source_path)
}

pub fn save_event_npc_refs(conn: &mut Connection, npc_refs: &[EventNpcRef]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_event_npc_ref.sql"))?;
        for npc in npc_refs {
            stmt.execute(params![
                npc.id,
                npc.npc_ini_id,
                npc.name,
                npc.role_description,
                i32::from(npc.role_result),
                npc.show_on_event,
                i32::from(npc.movement_mode),
                i32::from(npc.waypoint1_filled),
                npc.waypoint1_x,
                npc.waypoint1_y,
                npc.waypoint1_wait_time,
                i32::from(npc.waypoint1_facing_direction),
                npc.waypoint1_reserved,
                i32::from(npc.waypoint2_filled),
                npc.waypoint2_x,
                npc.waypoint2_y,
                npc.waypoint2_wait_time,
                i32::from(npc.waypoint2_facing_direction),
                npc.waypoint2_reserved,
                i32::from(npc.waypoint3_filled),
                npc.waypoint3_x,
                npc.waypoint3_y,
                npc.waypoint3_wait_time,
                i32::from(npc.waypoint3_facing_direction),
                npc.waypoint3_reserved,
                i32::from(npc.waypoint4_filled),
                npc.waypoint4_x,
                npc.waypoint4_y,
                npc.waypoint4_wait_time,
                i32::from(npc.waypoint4_facing_direction),
                npc.waypoint4_reserved,
                npc.activation_rect_x1,
                npc.activation_rect_y1,
                npc.activation_rect_x2,
                npc.activation_rect_y2,
                i32::from(npc.interaction_mode),
                npc.interaction_result,
                npc.interaction_range_offset,
                npc.dialog_id,
                npc.dialogue_face_sprite_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Build a well-formed row with 40 columns.
    fn row(id: i32, npc_ini_id: i32, name: &str, role: &str) -> String {
        let mut columns = vec![
            id.to_string(),
            npc_ini_id.to_string(),
            name.to_string(),
            role.to_string(),
            "0".to_string(), // role_result
            "0".to_string(), // show_on_event
            "1".to_string(), // movement_mode
        ];
        columns.extend(std::iter::repeat_n(
            "0".to_string(),
            COLUMN_COUNT - columns.len(),
        ));
        columns.join(",")
    }

    #[test]
    fn parse_full_row_reads_every_column() {
        let mut columns: Vec<String> = (0..COLUMN_COUNT).map(|i| i.to_string()).collect();
        columns[2] = "Lon".to_string();
        columns[3] = "awanturnik".to_string();
        columns[4] = "0".to_string(); // role_result
        columns[6] = "1".to_string(); // movement_mode
        columns[7] = "1".to_string(); // waypoint 1 filled
        columns[11] = "7".to_string(); // waypoint 1 facing direction
        columns[13] = "1".to_string(); // waypoint 2 filled
        columns[17] = "3".to_string(); // waypoint 2 facing direction
        columns[19] = "1".to_string(); // waypoint 3 filled
        columns[23] = "5".to_string(); // waypoint 3 facing direction
        columns[25] = "1".to_string(); // waypoint 4 filled
        columns[29] = "2".to_string(); // waypoint 4 facing direction
        columns[35] = "2".to_string(); // interaction mode
        let data = format!("{}\r\n", columns.join(","));

        let mut c = Cursor::new(data.as_bytes());
        let refs = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();

        assert_eq!(refs.len(), 1);
        let record = &refs[0];
        assert_eq!(record.id, 0);
        assert_eq!(record.npc_ini_id, 1);
        assert_eq!(record.name, "Lon");
        assert_eq!(record.role_description, "awanturnik");
        assert_eq!(record.role_result, NpcRoleResult::NormalDialogue);
        assert_eq!(record.show_on_event, 5);
        assert_eq!(record.movement_mode, NpcMovementMode::Waypoints);
        assert_eq!(record.waypoint1_filled, BooleanFlag::True);
        assert_eq!(record.waypoint1_x, 8);
        assert_eq!(record.waypoint1_y, 9);
        assert_eq!(record.waypoint1_wait_time, 10);
        assert_eq!(
            record.waypoint1_facing_direction,
            NpcLookingDirection::UpLeft
        );
        assert_eq!(record.waypoint2_filled, BooleanFlag::True);
        assert_eq!(
            record.waypoint2_facing_direction,
            NpcLookingDirection::DownRight
        );
        assert_eq!(
            record.waypoint3_facing_direction,
            NpcLookingDirection::DownLeft
        );
        assert_eq!(
            record.waypoint4_facing_direction,
            NpcLookingDirection::Right
        );
        assert_eq!(record.waypoint4_reserved, 30);
        assert_eq!(record.activation_rect_x1, 31);
        assert_eq!(record.activation_rect_y2, 34);
        assert_eq!(
            record.interaction_mode,
            NpcInteractionMode::ConfiguredThenRandom
        );
        assert_eq!(record.interaction_result, 36);
        assert_eq!(record.interaction_range_offset, 37);
        assert_eq!(record.dialog_id, 38);
        assert_eq!(record.dialogue_face_sprite_id, 39);
    }

    #[test]
    fn parse_skips_comments_and_empty_lines() {
        let data = format!("; comment\r\n\r\n{}\r\n", row(1, 61, "Lon", "awanturnik"));
        let mut c = Cursor::new(data.as_bytes());
        let refs = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "Lon");
    }

    #[test]
    fn parse_repairs_row_with_glued_id_and_name() {
        // The shipped file holds `2,69Ronin Galuc,przeor,...`: the comma
        // between the id and the name column is missing.
        let mut columns: Vec<String> = row(2, 69, "Ronin Galuc", "przeor")
            .split(',')
            .map(str::to_string)
            .collect();
        columns[1] = "69Ronin Galuc".to_string();
        columns.remove(2);
        assert_eq!(columns.len(), COLUMN_COUNT - 1);
        let data = format!("{}\r\n", columns.join(","));

        let mut c = Cursor::new(data.as_bytes());
        let refs = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, 2);
        assert_eq!(refs[0].npc_ini_id, 69);
        assert_eq!(refs[0].name, "Ronin Galuc");
        assert_eq!(refs[0].role_description, "przeor");
    }

    #[test]
    fn serialize_round_trip() {
        let data = format!(
            "{}\r\n{}\r\n",
            row(0, 61, "Lon", "awanturnik"),
            row(1, 9, "Rupaldis", "owiez")
        );
        let mut c = Cursor::new(data.as_bytes());
        let records = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        EventNpcRef::to_writer(&records, &mut out).unwrap();

        let mut c2 = Cursor::new(out.as_slice());
        let records2 = EventNpcRef::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].npc_ini_id, records2[0].npc_ini_id);
        assert_eq!(records[0].name, records2[0].name);
        assert_eq!(records[0].role_description, records2[0].role_description);
        assert_eq!(records[1].name, records2[1].name);
        assert_eq!(
            out.split(|b| *b == b'\n').count(),
            records.len() + 1,
            "every record is written on its own line"
        );
    }
}
