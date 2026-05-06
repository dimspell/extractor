// party_ref editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: party_ref,
    name_pascal: PartyRef,
    record: dispel_core::PartyRef,
    state_field: party_ref_editor,
    sheet_field: party_ref_spreadsheet,
    file: "Ref/PartyRef.ref",
}
