CREATE TABLE IF NOT EXISTS map_tiles (
    map_id TEXT,
    x INTEGER,
    y INTEGER,
    gtl_tile_id INTEGER,
    btl_tile_id INTEGER,
    collision BOOLEAN,
    event_id INTEGER,
    event_word INTEGER NOT NULL DEFAULT 0,
    marked BOOLEAN NOT NULL DEFAULT FALSE,
    object_id INTEGER,
    shadow_level INTEGER NOT NULL DEFAULT 0,
    light_flags INTEGER NOT NULL DEFAULT 0,
    access_ref_word INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (map_id, x, y)
);
CREATE INDEX IF NOT EXISTS idx_map_tiles_coords ON map_tiles(map_id, x, y);
