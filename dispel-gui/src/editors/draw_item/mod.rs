// draw_item editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: draw_item,
    name_pascal: DrawItem,
    record: dispel_core::DrawItem,
    state_field: draw_item_editor,
    sheet_field: draw_item_spreadsheet,
    file: "Ref/DRAWITEM.ref",
}
