// event_item editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: event_item,
    name_pascal: EventItem,
    record: dispel_core::EventItem,
    state_field: event_item_editor,
    sheet_field: event_item_spreadsheet,
    file: "CharacterInGame/EventItem.db",
}
