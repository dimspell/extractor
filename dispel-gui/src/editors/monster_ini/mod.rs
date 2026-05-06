// monster_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: monster_ini,
    name_pascal: MonsterIni,
    record: dispel_core::MonsterIni,
    state_field: monster_ini_editor,
    sheet_field: monster_ini_spreadsheet,
    file: "Monster.ini",
}
