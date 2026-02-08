CREATE TABLE IF NOT EXISTS map_metadata
(
    map_id          TEXT PRIMARY KEY,
    tiled_width     INTEGER,
    tiled_height    INTEGER,
    width_pixels    INTEGER,
    height_pixels   INTEGER,
    non_occluded_x  INTEGER,
    non_occluded_y  INTEGER,
    occluded_width  INTEGER,
    occluded_height INTEGER
)
