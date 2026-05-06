// npc_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: npc_ini,
    name_pascal: NpcIni,
    record: dispel_core::NpcIni,
    state_field: npc_ini_editor,
    sheet_field: npc_ini_spreadsheet,
    file: "Npc.ini",
}
