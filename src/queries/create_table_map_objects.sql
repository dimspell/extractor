CREATE TABLE IF NOT EXISTS map_objects (
    map_id TEXT,
    object_index INTEGER,
    x INTEGER,
    y INTEGER,
    btl_tile_id INTEGER,
    stack_order INTEGER,
    PRIMARY KEY (map_id, object_index, stack_order)
);
CREATE INDEX IF NOT EXISTS idx_map_objects_map_id ON map_objects(map_id);
