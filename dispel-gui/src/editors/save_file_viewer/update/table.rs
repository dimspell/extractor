use std::collections::HashMap;

use crate::editors::save_file_viewer::message::TableKey;
use crate::editors::save_file_viewer::state::{
    InventoryCategory, JournalSection, MapsDisplayCaches, MapsTableKind, SaveFileViewerState,
};

/// Apply a single cursor-move event to whichever table resize is active.
/// Narrows the unified `state.resizing` to the relevant table state.
pub fn apply_resize_cursor(state: &mut SaveFileViewerState, x: f32) {
    let drag = match state.resizing.as_mut() {
        Some(d) => d,
        None => return,
    };
    let anchor_x = match drag.anchor_cursor_x {
        Some(ax) => ax,
        None => {
            drag.anchor_cursor_x = Some(x);
            return;
        }
    };
    let new_width = (drag.anchor_width + (x - anchor_x)).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);

    match drag.key {
        TableKey::Map(map, kind) => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
                && let Some(w) = ts.column_widths.get_mut(drag.col)
            {
                *w = new_width;
            }
        }
        TableKey::Inventory(cat) => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat)
                && let Some(w) = ts.column_widths.get_mut(drag.col)
            {
                *w = new_width;
            }
        }
        TableKey::Events => {
            if let Some(w) = state.events_table_state.column_widths.get_mut(drag.col) {
                *w = new_width;
            }
        }
        TableKey::Journal(section) => {
            if let Some(ts) = state.journal_table_states.get_mut(&section)
                && let Some(w) = ts.column_widths.get_mut(drag.col)
            {
                *w = new_width;
            }
        }
    }
}

/// Render raw bytes as uppercase, space-separated hex (e.g. "DE AD BE EF").
pub fn hex_bytes(v: &[u8]) -> String {
    v.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Clamp bounds for column resize widths.
pub const COL_WIDTH_MIN: f32 = 24.0;
pub const COL_WIDTH_MAX: f32 = 600.0;

/// Return the (immutable) indices slice for a given map table kind.
pub fn maps_table_indices(cache: &MapsDisplayCaches, kind: MapsTableKind) -> &[usize] {
    match kind {
        MapsTableKind::Monsters => &cache.monsters_indices,
        MapsTableKind::Npcs => &cache.npcs_indices,
        MapsTableKind::ExtraObjects => &cache.extra_objects_indices,
        MapsTableKind::Weapon => &cache.draw_items_weapon_indices,
        MapsTableKind::Heal => &cache.draw_items_heal_indices,
        MapsTableKind::Edit => &cache.draw_items_edit_indices,
        MapsTableKind::Misc => &cache.draw_items_misc_indices,
        MapsTableKind::Event => &cache.draw_items_event_indices,
    }
}

/// Return the (immutable rows, mutable indices) pair for a given map table
/// kind. The two borrows are disjoint fields of `MapsDisplayCaches`.
pub fn maps_table_data(
    cache: &mut MapsDisplayCaches,
    kind: MapsTableKind,
) -> (&[Vec<String>], &mut Vec<usize>) {
    match kind {
        MapsTableKind::Monsters => (&cache.monsters, &mut cache.monsters_indices),
        MapsTableKind::Npcs => (&cache.npcs, &mut cache.npcs_indices),
        MapsTableKind::ExtraObjects => (&cache.extra_objects, &mut cache.extra_objects_indices),
        MapsTableKind::Weapon => (
            &cache.draw_items_weapon,
            &mut cache.draw_items_weapon_indices,
        ),
        MapsTableKind::Heal => (&cache.draw_items_heal, &mut cache.draw_items_heal_indices),
        MapsTableKind::Edit => (&cache.draw_items_edit, &mut cache.draw_items_edit_indices),
        MapsTableKind::Misc => (&cache.draw_items_misc, &mut cache.draw_items_misc_indices),
        MapsTableKind::Event => (&cache.draw_items_event, &mut cache.draw_items_event_indices),
    }
}

/// Return the (immutable rows, mutable indices) pair for a given inventory
/// category. The two borrows are disjoint fields of the two HashMaps.
pub fn inventory_table_data<'a>(
    cache: &'a mut HashMap<InventoryCategory, Vec<Vec<String>>>,
    indices: &'a mut HashMap<InventoryCategory, Vec<usize>>,
    cat: InventoryCategory,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    let rows = cache.get(&cat).map(|v| &v[..]).unwrap_or(&[]);
    let idx = indices.get_mut(&cat).expect("inventory indices missing");
    (rows, idx)
}

/// Return the (immutable rows, mutable indices) pair for the events table.
pub fn events_table_data<'a>(
    cache: &'a mut [Vec<String>],
    indices: &'a mut Vec<usize>,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    (&cache[..], indices)
}

/// Return the (immutable rows, mutable indices) pair for a journal table.
pub fn journal_table_data<'a>(
    cache: &'a mut HashMap<JournalSection, Vec<Vec<String>>>,
    indices: &'a mut HashMap<JournalSection, Vec<usize>>,
    section: JournalSection,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    let rows = cache.get(&section).map(|v| &v[..]).unwrap_or(&[]);
    let idx = indices.get_mut(&section).expect("journal indices missing");
    (rows, idx)
}

/// Auto-size a column to fit the longest visible cell value plus the column
/// header label, clamped to [`COL_WIDTH_MIN`, `COL_WIDTH_MAX`].
///
/// Uses a fixed per-character width of 7px + 16px padding, matching the
/// standard spreadsheet editor's [`SpreadsheetState::auto_size_column`].
pub fn auto_size_column(rows: &[Vec<String>], indices: &[usize], col: usize, header: &str) -> f32 {
    let mut max_chars = header.chars().count().max(1);
    for &idx in indices {
        if let Some(cell) = rows.get(idx).and_then(|r| r.get(col)) {
            let chars = cell.chars().count();
            if chars > max_chars {
                max_chars = chars;
            }
        }
    }
    ((max_chars as f32) * 7.0 + 16.0).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX)
}

/// Return the display rows slice for a given map table kind (immutable).
pub fn maps_table_rows(cache: &MapsDisplayCaches, kind: MapsTableKind) -> &[Vec<String>] {
    match kind {
        MapsTableKind::Monsters => &cache.monsters,
        MapsTableKind::Npcs => &cache.npcs,
        MapsTableKind::ExtraObjects => &cache.extra_objects,
        MapsTableKind::Weapon => &cache.draw_items_weapon,
        MapsTableKind::Heal => &cache.draw_items_heal,
        MapsTableKind::Edit => &cache.draw_items_edit,
        MapsTableKind::Misc => &cache.draw_items_misc,
        MapsTableKind::Event => &cache.draw_items_event,
    }
}

#[cfg(test)]
mod tests {
    use super::hex_bytes;

    #[test]
    fn hex_bytes_empty() {
        assert_eq!(hex_bytes(&[]), "");
    }

    #[test]
    fn hex_bytes_single_byte() {
        assert_eq!(hex_bytes(&[0xDE]), "DE");
    }

    #[test]
    fn hex_bytes_multiple_bytes() {
        assert_eq!(hex_bytes(&[0xDE, 0xAD, 0xBE, 0xEF]), "DE AD BE EF");
    }

    #[test]
    fn hex_bytes_leading_zero() {
        assert_eq!(hex_bytes(&[0x00, 0xFF, 0x01]), "00 FF 01");
    }
}
