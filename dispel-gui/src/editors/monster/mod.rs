// monster editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: monster,
    name_pascal: Monster,
    record: dispel_core::Monster,
    state_field: monster_editor,
    sheet_field: monster_spreadsheet,
    file: "MonsterInGame/Monster.db",
}
