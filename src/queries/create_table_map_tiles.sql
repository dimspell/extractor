CREATE TABLE IF NOT EXISTS map_tiles (
    map_id TEXT,
    x INTEGER,
    y INTEGER,
    gtl_tile_id INTEGER,
    btl_tile_id INTEGER,
    collision BOOLEAN,
    event_id INTEGER,
    PRIMARY KEY (map_id, x, y)
);
CREATE INDEX IF NOT EXISTS idx_map_tiles_coords ON map_tiles(map_id, x, y);
