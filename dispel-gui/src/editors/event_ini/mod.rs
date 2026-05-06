// event_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: event_ini,
    name_pascal: EventIni,
    record: dispel_core::Event,
    state_field: event_ini_editor,
    sheet_field: event_ini_spreadsheet,
    file: "Event.ini",
}
