// misc_item editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: misc_item,
    name_pascal: MiscItem,
    record: dispel_core::MiscItem,
    state_field: misc_item_editor,
    sheet_field: misc_item_spreadsheet,
    file: "CharacterInGame/MiscItem.db",
}
