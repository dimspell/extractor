// edit_item editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: edit_item,
    name_pascal: EditItem,
    record: dispel_core::EditItem,
    state_field: edit_item_editor,
    sheet_field: edit_item_spreadsheet,
    file: "CharacterInGame/EditItem.db",
}
