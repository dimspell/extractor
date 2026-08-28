use crate::components::editable::EditableRecord;
use crate::editable_record_fields;
use dispel_core::TileGrid;

editable_record_fields!(TileGrid, {
    { id = Integer / "ID:" },
    { payload_length = Integer / "Payload Length:" },
    { grid_size = Integer / "Grid Size:" },
});

impl EditableRecord for TileGrid {
    crate::editable_record_delegate!();
    fn list_label(&self) -> String {
        format!(
            "Tile grid (grid_size={}, entries={})",
            self.grid_size,
            self.data.len()
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
