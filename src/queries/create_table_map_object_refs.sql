CREATE TABLE IF NOT EXISTS map_object_refs (
    map_id TEXT NOT NULL,
    ref_index INTEGER NOT NULL,
    word0 INTEGER NOT NULL,
    word1 INTEGER NOT NULL,
    PRIMARY KEY (map_id, ref_index)
);
