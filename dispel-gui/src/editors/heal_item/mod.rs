// heal_item editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: heal_item,
    name_pascal: HealItem,
    record: dispel_core::HealItem,
    state_field: heal_item_editor,
    sheet_field: heal_item_spreadsheet,
    file: "CharacterInGame/healItem.db",
}
