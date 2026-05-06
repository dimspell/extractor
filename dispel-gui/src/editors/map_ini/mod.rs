// map_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: map_ini,
    name_pascal: MapIni,
    record: dispel_core::MapIni,
    state_field: map_ini_editor,
    sheet_field: map_ini_spreadsheet,
    file: "Ref/Map.ini",
}
