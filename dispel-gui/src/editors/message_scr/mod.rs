// message_scr editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: message_scr,
    name_pascal: MessageScr,
    record: dispel_core::Message,
    state_field: message_scr_editor,
    sheet_field: message_scr_spreadsheet,
    file: "ExtraInGame/Message.scr",
}
