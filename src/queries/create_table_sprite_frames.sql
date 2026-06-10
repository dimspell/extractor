CREATE TABLE IF NOT EXISTS sprite_frames (
    sprite_file_id INTEGER NOT NULL REFERENCES sprite_files(id),
    sequence_index INTEGER NOT NULL,
    frame_index INTEGER NOT NULL,
    png_blob BLOB NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    origin_x INTEGER NOT NULL,
    origin_y INTEGER NOT NULL,
    PRIMARY KEY (sprite_file_id, sequence_index, frame_index)
);
