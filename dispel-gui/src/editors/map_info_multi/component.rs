use crate::components::editable::EditableRecord;
use crate::editable_record_fields;
use dispel_core::MapInfoMulti;

editable_record_fields!(MapInfoMulti, {
    { id = Integer / "ID:" },
    { self_name = String / "Self Name:" },
    { anchor_tile_index = Integer / "Anchor Tile:" },
    { stride = Integer / "Stride:" },
    { row_step = Integer / "Row Step:" },
    { start_pos_a_x = Integer / "Start A X:" },
    { start_pos_a_y = Integer / "Start A Y:" },
    { start_pos_b_x = Integer / "Start B X:" },
    { start_pos_b_y = Integer / "Start B Y:" },
    { transition_target = String / "Transition Target:" },
    { mon_filename = String / "Mon Filename:" },
});

impl EditableRecord for MapInfoMulti {
    crate::editable_record_delegate!();
    fn list_label(&self) -> String {
        format!(
            "Map info (name={}, anchor={}, stride={})",
            self.self_name, self.anchor_tile_index, self.stride
        )
    }
    fn detail_title() -> &'static str {
        "Map Info Multi Details"
    }
    fn empty_selection_text() -> &'static str {
        "No map info selected"
    }
    fn save_button_label() -> &'static str {
        "Save Map Info"
    }
}
