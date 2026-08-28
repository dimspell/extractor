mod component;

crate::define_standard_editor! {
    name: tile_grid,
    name_pascal: TileGrid,
    record: dispel_core::TileGrid,
    field: tile_grid_editor,
    file: "Map/Multi/*.dat",
}
