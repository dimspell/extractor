CREATE TABLE IF NOT EXISTS map_bundles (
    map_id TEXT NOT NULL,
    bundle_index INTEGER NOT NULL,
    PRIMARY KEY (map_id, bundle_index)
);
