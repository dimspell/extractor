use crate::editors::save_file_viewer::message::TableKey;
use crate::editors::save_file_viewer::state::{JournalSection, MapsTableKind, SaveFileViewerState};

/// Build a default filename for a CSV export based on the table key.
pub fn csv_default_filename(key: TableKey) -> String {
    match key {
        TableKey::Inventory(cat) => format!("inventory-{}.csv", cat.label()),
        TableKey::Events => "events.csv".to_string(),
        TableKey::Journal(section) => {
            let label = match section {
                JournalSection::Main => "main",
                JournalSection::Side => "side",
                JournalSection::Trade => "trade",
            };
            format!("journal-{label}.csv")
        }
        TableKey::Map(_, kind) => {
            let label = match kind {
                MapsTableKind::Monsters => "monsters",
                MapsTableKind::Npcs => "npcs",
                MapsTableKind::ExtraObjects => "extra-objects",
                MapsTableKind::Weapon => "weapons",
                MapsTableKind::Heal => "heals",
                MapsTableKind::Edit => "edits",
                MapsTableKind::Misc => "misc",
                MapsTableKind::Event => "events",
            };
            format!("map-{label}.csv")
        }
    }
}

/// Resolve (column headers, filtered rows) for a table identified by `key`.
/// Returns `None` when the table has no data (empty cache or missing state).
pub fn resolve_csv_export_data(
    state: &SaveFileViewerState,
    key: TableKey,
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    match key {
        TableKey::Inventory(cat) => {
            let headers: Vec<String> = cat
                .default_columns()
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let cache = state.inventory_display_caches.get(&cat)?;
            let indices = state.inventory_filtered_indices.get(&cat)?;
            let rows: Vec<Vec<String>> = indices
                .iter()
                .filter_map(|&i| cache.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
        TableKey::Events => {
            let headers: Vec<String> =
                crate::editors::save_file_viewer::state::events_default_columns()
                    .iter()
                    .map(|c| c.label.clone())
                    .collect();
            let rows: Vec<Vec<String>> = state
                .events_filtered_indices
                .iter()
                .filter_map(|&i| state.events_display_cache.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
        TableKey::Journal(section) => {
            let headers: Vec<String> = section
                .default_columns()
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let cache = state.journal_display_caches.get(&section)?;
            let indices = state.journal_filtered_indices.get(&section)?;
            let rows: Vec<Vec<String>> = indices
                .iter()
                .filter_map(|&i| cache.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
        TableKey::Map(map_idx, kind) => {
            let headers: Vec<String> = kind
                .default_columns()
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let cache = state.maps_display_caches.get(map_idx)?;
            let (rows_data, indices_slice): (&[Vec<String>], &[usize]) = match kind {
                MapsTableKind::Monsters => (&cache.monsters, &cache.monsters_indices),
                MapsTableKind::Npcs => (&cache.npcs, &cache.npcs_indices),
                MapsTableKind::ExtraObjects => (&cache.extra_objects, &cache.extra_objects_indices),
                MapsTableKind::Weapon => {
                    (&cache.draw_items_weapon, &cache.draw_items_weapon_indices)
                }
                MapsTableKind::Heal => (&cache.draw_items_heal, &cache.draw_items_heal_indices),
                MapsTableKind::Edit => (&cache.draw_items_edit, &cache.draw_items_edit_indices),
                MapsTableKind::Misc => (&cache.draw_items_misc, &cache.draw_items_misc_indices),
                MapsTableKind::Event => (&cache.draw_items_event, &cache.draw_items_event_indices),
            };
            let rows: Vec<Vec<String>> = indices_slice
                .iter()
                .filter_map(|&i| rows_data.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{csv_default_filename, JournalSection, MapsTableKind, TableKey};

    #[test]
    fn csv_filename_inventory() {
        let name = csv_default_filename(TableKey::Inventory(
            crate::editors::save_file_viewer::state::InventoryCategory::Weapon,
        ));
        assert!(name.contains("inventory"));
        assert!(name.ends_with(".csv"));
    }

    #[test]
    fn csv_filename_events() {
        assert_eq!(csv_default_filename(TableKey::Events), "events.csv");
    }

    #[test]
    fn csv_filename_journal() {
        let name = csv_default_filename(TableKey::Journal(JournalSection::Main));
        assert_eq!(name, "journal-main.csv");
    }

    #[test]
    fn csv_filename_map_monsters() {
        let name = csv_default_filename(TableKey::Map(0, MapsTableKind::Monsters));
        assert_eq!(name, "map-monsters.csv");
    }
}
