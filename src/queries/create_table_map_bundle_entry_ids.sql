CREATE TABLE IF NOT EXISTS map_bundle_entry_ids (
    map_id TEXT NOT NULL,
    bundle_index INTEGER NOT NULL,
    record_index INTEGER NOT NULL,
    item_index INTEGER NOT NULL,
    entry_index INTEGER NOT NULL,
    seq INTEGER NOT NULL,
    id INTEGER NOT NULL,
    PRIMARY KEY (map_id, bundle_index, record_index, item_index, entry_index, seq)
);
