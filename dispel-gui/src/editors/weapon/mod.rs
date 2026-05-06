// weapon editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: weapon,
    name_pascal: Weapon,
    record: dispel_core::WeaponItem,
    state_field: weapon_editor,
    sheet_field: weapon_spreadsheet,
    file: "CharacterInGame/weaponItem.db",
}
