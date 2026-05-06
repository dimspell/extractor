// party_ini editor module — generated via define_standard_editor!

mod component;

crate::define_standard_editor! {
    name: party_ini,
    name_pascal: PartyIni,
    record: dispel_core::PartyIniNpc,
    field: party_ini_editor,
    file: "NpcInGame/PrtIni.db",
}
