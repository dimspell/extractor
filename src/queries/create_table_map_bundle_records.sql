CREATE TABLE IF NOT EXISTS map_bundle_records (
    map_id TEXT NOT NULL,
    bundle_index INTEGER NOT NULL,
    record_index INTEGER NOT NULL,
    field_04 INTEGER NOT NULL,
    body BLOB NOT NULL,
    item_count INTEGER NOT NULL,
    PRIMARY KEY (map_id, bundle_index, record_index)
);
