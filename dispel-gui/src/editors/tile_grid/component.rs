use crate::components::editable::EditableRecord;
use crate::editable_record_fields;
use dispel_core::TileGrid;
use dispel_core::references::tile_grid::TileDescriptor;

editable_record_fields!(TileGrid, {
    { id = Integer / "ID:" },
    { payload_length = Integer / "Payload Length:" },
    { grid_size = Integer / "Grid Size:" },
    { data = VecU32 / "Grid Data (comma-separated u32):" },
});

impl EditableRecord for TileGrid {
    crate::editable_record_delegate!();
    fn list_label(&self) -> String {
        let stats = grid_stats(self);
        format!(
            "Tile grid (grid={}x{}, entries={}, walls={}, empty={}, unique_tiles={})",
            self.grid_size,
            self.grid_size,
            self.data.len(),
            stats.walls,
            stats.empty,
            stats.unique_tiles
        )
    }
    fn detail_title() -> &'static str {
        "Tile Grid Details"
    }
    fn empty_selection_text() -> &'static str {
        "No tile grid selected"
    }
    fn save_button_label() -> &'static str {
        "Save Tile Grid"
    }
}

/// Summary statistics for a tile grid, computed via `TileDescriptor` unpacking.
struct GridStats {
    /// Tiles with `object_id == 1` (walls).
    walls: usize,
    /// Tiles with `object_id == 0` (empty).
    empty: usize,
    /// Number of distinct `tile_index` values across the grid.
    unique_tiles: usize,
}

fn grid_stats(grid: &TileGrid) -> GridStats {
    let mut walls = 0usize;
    let mut empty = 0usize;
    let mut tile_indices = std::collections::HashSet::new();
    for word in &grid.data {
        let d = TileDescriptor::from_word(*word);
        match d.object_id {
            0 => empty += 1,
            1 => walls += 1,
            _ => {}
        }
        tile_indices.insert(d.tile_index);
    }
    GridStats {
        walls,
        empty,
        unique_tiles: tile_indices.len(),
    }
}
