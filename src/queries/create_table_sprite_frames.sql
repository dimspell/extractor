CREATE TABLE IF NOT EXISTS sprite_frames (
    normalized_path TEXT NOT NULL,
    sequence_index INTEGER NOT NULL,
    frame_index INTEGER NOT NULL,
    png_blob BLOB NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    origin_x INTEGER NOT NULL,
    origin_y INTEGER NOT NULL,
    PRIMARY KEY (normalized_path, sequence_index, frame_index)
);
