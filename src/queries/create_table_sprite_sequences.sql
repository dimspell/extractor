CREATE TABLE IF NOT EXISTS sprite_sequences (
    normalized_path TEXT NOT NULL,
    sequence_index INTEGER NOT NULL,
    frame_count INTEGER NOT NULL,
    first_frame_width INTEGER NOT NULL,
    first_frame_height INTEGER NOT NULL,
    first_frame_origin_x INTEGER NOT NULL,
    first_frame_origin_y INTEGER NOT NULL,
    PRIMARY KEY (normalized_path, sequence_index)
);
