// chdata editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: chdata,
    name_pascal: ChData,
    record: dispel_core::ChData,
    state_field: chdata_editor,
    sheet_field: chdata_spreadsheet,
    file: "CharacterInGame/ChData.db",
}
