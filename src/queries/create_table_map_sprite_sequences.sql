CREATE TABLE IF NOT EXISTS map_sprite_sequences (
    map_id TEXT NOT NULL,
    sequence_index INTEGER NOT NULL,
    start_position INTEGER NOT NULL,
    end_position INTEGER NOT NULL,
    frame_count INTEGER NOT NULL,
    PRIMARY KEY (map_id, sequence_index)
);
