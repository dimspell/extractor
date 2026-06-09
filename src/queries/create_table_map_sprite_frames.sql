CREATE TABLE IF NOT EXISTS map_sprite_frames (
    map_id TEXT NOT NULL,
    internal_sprite_id INTEGER NOT NULL,
    frame_index INTEGER NOT NULL,
    png_blob BLOB NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    origin_x INTEGER NOT NULL,
    origin_y INTEGER NOT NULL,
    PRIMARY KEY (map_id, internal_sprite_id, frame_index)
);
