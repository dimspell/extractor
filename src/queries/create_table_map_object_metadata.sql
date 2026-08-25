CREATE TABLE IF NOT EXISTS map_object_metadata (
    map_id TEXT NOT NULL,
    object_index INTEGER NOT NULL,
    metadata_blob BLOB NOT NULL,
    control_0 INTEGER NOT NULL,
    control_1 INTEGER NOT NULL,
    control_2 INTEGER NOT NULL,
    control_3 INTEGER NOT NULL,
    param_0 INTEGER NOT NULL,
    param_1 INTEGER NOT NULL,
    param_2 INTEGER NOT NULL,
    param_3 INTEGER NOT NULL,
    param_4 INTEGER NOT NULL,
    param_5 INTEGER NOT NULL,
    extra_count_a INTEGER NOT NULL,
    extra_count_b INTEGER NOT NULL,
    PRIMARY KEY (map_id, object_index)
);
