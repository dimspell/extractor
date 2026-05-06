// weapon editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: weapon,
    name_pascal: Weapon,
    record: dispel_core::WeaponItem,
    field: weapon_editor,
    file: "CharacterInGame/weaponItem.db",
}
