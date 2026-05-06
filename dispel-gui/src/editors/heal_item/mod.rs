// heal_item editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: heal_item,
    name_pascal: HealItem,
    record: dispel_core::HealItem,
    field: heal_item_editor,
    file: "CharacterInGame/healItem.db",
}
