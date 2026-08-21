use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub(super) const EVENT_COUNT: usize = 2_251;
pub(super) const EVENT_RECORD_SIZE: usize = 284;
pub(super) const POST_EVENTS_RECORD_SIZE: usize = 24;
pub(super) const RECRUITABLE_COMPANION_COUNT: usize = 8;

/// Event-script definition and its saved runtime state (284 bytes).
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct EventRecord {
    /// Event identifier and index in the fixed event table.
    pub event_id: u32,
    /// Event whose triggered state controls this event for types `3`-`8`.
    pub required_event_id: u32,
    /// Dispatch rule:
    ///
    /// - `0` = once, unconditionally
    /// - `1` = up to `execution_limit` times, unconditionally
    /// - `2` = always, unconditionally
    /// - `3` = once, while the required event has not triggered
    /// - `4` = up to `execution_limit` times, while the required event has not triggered
    /// - `5` = always, while the required event has not triggered
    /// - `6` = once, after the required event has triggered
    /// - `7` = up to `execution_limit` times, after the required event has triggered
    /// - `8` = always, after the required event has triggered
    pub event_type: u32,
    /// Event script filename, or an empty string when no script is assigned.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 260))]
    pub script_filename: String,
    /// Maximum trigger count used by event types `1`, `4`, and `7`.
    pub execution_limit: u32,
    /// Number of times this event has started dispatching.
    pub execution_count: u32,
    /// Whether this event has started dispatching (`0`=not triggered, `1`=triggered).
    pub has_triggered: u32,
}

pub(super) fn read_events<R: Read>(reader: &mut R) -> std::io::Result<Vec<EventRecord>> {
    let mut events = Vec::with_capacity(EVENT_COUNT);
    for _ in 0..EVENT_COUNT {
        let mut data = [0u8; EVENT_RECORD_SIZE];
        reader.read_exact(&mut data)?;
        events.push(EventRecord::parse(&data)?);
    }
    Ok(events)
}

pub(super) fn write_events<W: Write>(
    events: &[EventRecord],
    writer: &mut W,
) -> std::io::Result<()> {
    for event in events {
        event.write(writer)?;
    }
    Ok(())
}

/// Data block between events and journal sections.
///
/// Screen effects and movement-event history stored after the event scripts.
///
/// The two movement collections contain fixed 24-byte records. Completion
/// records are also used to detect when a character reaches a position where
/// an earlier walk ended.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostEventsData {
    /// Whether a screen-shake effect is active (`0`=inactive, `1`=active).
    pub shake_active: u32,
    /// Number of frames remaining in the active screen-shake effect.
    pub shake_frames_remaining: u32,
    /// Movement events emitted at animation or path-progress milestones.
    pub walk_milestones: Vec<WalkMilestoneRecord>,
    /// Movement events emitted when a walk cycle finishes.
    pub walk_completions: Vec<WalkCompletionRecord>,
    /// World-presence state for each recruitable companion.
    ///
    /// `0` means the companion was removed from the map, including when the
    /// companion joined the party. `1` means the companion remains available
    /// in the world.
    pub recruitable_companion_world_presence: [u32; RECRUITABLE_COMPANION_COUNT],
    /// Retained progression for companions that previously left the party.
    pub dismissed_companion_progression:
        [DismissedCompanionProgression; RECRUITABLE_COMPANION_COUNT],
}

/// Progression retained when a companion leaves the active party.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DismissedCompanionProgression {
    /// Whether retained progression exists (`0`=none, `1`=present).
    pub is_saved: u8,
    /// Companion level when the progression was retained.
    pub companion_level: u8,
    /// Player level when the progression was retained.
    pub player_level: u8,
}

impl PostEventsData {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let shake_active = reader.read_u32::<LittleEndian>()?;
        let shake_frames_remaining = reader.read_u32::<LittleEndian>()?;

        let walk_milestones = read_records(reader, WalkMilestoneRecord::parse)?;
        let walk_completions = read_records(reader, WalkCompletionRecord::parse)?;

        let mut recruitable_companion_world_presence = [0u32; RECRUITABLE_COMPANION_COUNT];
        for presence in &mut recruitable_companion_world_presence {
            *presence = reader.read_u32::<LittleEndian>()?;
        }

        let mut dismissed_companion_progression =
            [DismissedCompanionProgression::default(); RECRUITABLE_COMPANION_COUNT];
        for progression in &mut dismissed_companion_progression {
            progression.is_saved = reader.read_u8()?;
            progression.companion_level = reader.read_u8()?;
            progression.player_level = reader.read_u8()?;
        }

        Ok(Self {
            shake_active,
            shake_frames_remaining,
            walk_milestones,
            walk_completions,
            recruitable_companion_world_presence,
            dismissed_companion_progression,
        })
    }

    pub(super) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.shake_active)?;
        writer.write_u32::<LittleEndian>(self.shake_frames_remaining)?;

        write_records(writer, &self.walk_milestones, |record, writer| {
            record.write(writer)
        })?;
        write_records(writer, &self.walk_completions, |record, writer| {
            record.write(writer)
        })?;

        for presence in &self.recruitable_companion_world_presence {
            writer.write_u32::<LittleEndian>(*presence)?;
        }

        for progression in &self.dismissed_companion_progression {
            writer.write_u8(progression.is_saved)?;
            writer.write_u8(progression.companion_level)?;
            writer.write_u8(progression.player_level)?;
        }
        Ok(())
    }
}

/// A single 24-byte walk milestone record.
///
/// Created when walk progress approaches the end of a path or an animation
/// reaches its movement milestone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, BinaryRecord)]
pub struct WalkMilestoneRecord {
    /// Event id. `400`; `10`/`100`/`200`/`300` when the
    /// walk-freshness counter is active; ascending global counter in shipped
    /// saves (53, 66, 73, ...).
    pub id: u32,
    /// Walk direction (animation-step direction, 0-7).
    pub direction: u32,
    /// Character movement state (`0`=idle, `1`=walking).
    pub state: u32,
    /// Walk-type flag. `0` (which duplicates `direction`
    /// into this slot); `1` in shipped saves.
    pub walk_type: u32,
    /// X coordinate.
    pub x: u32,
    /// Y coordinate.
    pub y: u32,
}

/// A single 24-byte walk completion record.
///
/// The record is created when the active path reaches its completion point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, BinaryRecord)]
pub struct WalkCompletionRecord {
    /// Event id. `2000`; ascending global counter in
    /// shipped saves (684, 775, 828, ...).
    pub id: u32,
    /// Normalized walk direction (0-3; walk directions 4-7 map to 0-3).
    pub direction: u32,
    /// Diagonal flag: `1` when the walk direction is diagonal.
    pub diagonal: u32,
    /// Character index (`0` for party members; `0`-`2` in
    /// shipped saves).
    pub character_index: u32,
    /// X coordinate.
    pub x: u32,
    /// Y coordinate.
    pub y: u32,
}

fn read_records<R: Read, T>(
    reader: &mut R,
    parse: impl Fn(&[u8]) -> std::io::Result<T>,
) -> std::io::Result<Vec<T>> {
    let count = reader.read_u32::<LittleEndian>()? as usize;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let mut data = [0u8; POST_EVENTS_RECORD_SIZE];
        reader.read_exact(&mut data)?;
        records.push(parse(&data)?);
    }
    Ok(records)
}

fn write_records<W: Write, T>(
    writer: &mut W,
    records: &[T],
    write: impl Fn(&T, &mut W) -> std::io::Result<()>,
) -> std::io::Result<()> {
    writer.write_u32::<LittleEndian>(records.len() as u32)?;
    for record in records {
        write(record, writer)?;
    }
    Ok(())
}
