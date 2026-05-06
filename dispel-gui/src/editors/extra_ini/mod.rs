// extra_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: extra_ini,
    name_pascal: ExtraIni,
    record: dispel_core::Extra,
    state_field: extra_ini_editor,
    sheet_field: extra_ini_spreadsheet,
    file: "Extra.ini",
}
