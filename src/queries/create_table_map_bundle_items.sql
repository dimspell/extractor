CREATE TABLE IF NOT EXISTS map_bundle_items (
    map_id TEXT NOT NULL,
    bundle_index INTEGER NOT NULL,
    record_index INTEGER NOT NULL,
    item_index INTEGER NOT NULL,
    type_flag INTEGER NOT NULL,
    field_14 INTEGER NOT NULL,
    entry_count INTEGER NOT NULL,
    PRIMARY KEY (map_id, bundle_index, record_index, item_index)
);
