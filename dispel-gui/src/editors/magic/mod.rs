// magic editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: magic,
    name_pascal: Magic,
    record: dispel_core::MagicSpell,
    state_field: magic_editor,
    sheet_field: magic_spreadsheet,
    file: "MagicInGame/Magic.db",
}
