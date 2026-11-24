CREATE TABLE IF NOT EXISTS map_bundle_flags (
    map_id TEXT NOT NULL,
    bundle_index INTEGER NOT NULL,
    level_index INTEGER NOT NULL,
    flag_a INTEGER NOT NULL,
    flag_b INTEGER NOT NULL,
    PRIMARY KEY (map_id, bundle_index, level_index)
);
