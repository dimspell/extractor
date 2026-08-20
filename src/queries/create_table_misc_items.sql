CREATE TABLE IF NOT EXISTS misc_items
(
    id           INTEGER PRIMARY KEY,
    name         TEXT,
    description  TEXT,
    base_price   INTEGER,
    reserved_bytes BLOB,
    runtime_record_index_slot INTEGER
);
