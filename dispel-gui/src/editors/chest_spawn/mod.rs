mod component;

crate::define_standard_editor! {
    name: chest_spawn,
    name_pascal: ChestSpawn,
    record: dispel_core::ChestSpawn,
    field: chest_spawn_editor,
    file: "Map/Multi/*.cst",
}
