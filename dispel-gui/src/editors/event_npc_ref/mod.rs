// event_npc_ref editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: event_npc_ref,
    name_pascal: EventNpcRef,
    record: dispel_core::EventNpcRef,
    state_field: event_npc_ref_editor,
    sheet_field: event_npc_ref_spreadsheet,
    file: "NpcInGame/Eventnpc.ref",
}
