// all_map_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: all_map_ini,
    name_pascal: AllMapIni,
    record: dispel_core::Map,
    state_field: all_map_ini_editor,
    sheet_field: all_map_ini_spreadsheet,
    file: "AllMap.ini",
}
