CREATE TABLE IF NOT EXISTS sprite_sequences (
    sprite_file_id INTEGER NOT NULL REFERENCES sprite_files(id),
    sequence_index INTEGER NOT NULL,
    frame_count INTEGER NOT NULL,
    first_frame_width INTEGER NOT NULL,
    first_frame_height INTEGER NOT NULL,
    first_frame_origin_x INTEGER NOT NULL,
    first_frame_origin_y INTEGER NOT NULL,
    PRIMARY KEY (sprite_file_id, sequence_index)
);
