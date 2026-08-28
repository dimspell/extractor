mod component;

crate::define_standard_editor! {
    name: monster_spawn,
    name_pascal: MonsterSpawn,
    record: dispel_core::MonsterSpawn,
    field: monster_spawn_editor,
    file: "Map/Multi/*.mon",
}
