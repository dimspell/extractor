// quest_scr editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: quest_scr,
    name_pascal: QuestScr,
    record: dispel_core::Quest,
    state_field: quest_scr_editor,
    sheet_field: quest_scr_spreadsheet,
    file: "ExtraInGame/Quest.scr",
}
