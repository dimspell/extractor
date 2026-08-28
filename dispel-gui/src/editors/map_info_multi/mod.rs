mod component;

crate::define_standard_editor! {
    name: map_info_multi,
    name_pascal: MapInfoMulti,
    record: dispel_core::MapInfoMulti,
    field: map_info_multi_editor,
    file: "Map/Multi/*.ifm",
}
