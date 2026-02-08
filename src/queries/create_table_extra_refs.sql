CREATE TABLE IF NOT EXISTS extra_refs
(
    file_path              TEXT,
    id                     INTEGER,
    number_in_file         INTEGER,
    ext_id                 INTEGER,
    name                   TEXT,
    object_type            INTEGER,
    x_pos                  INTEGER,
    y_pos                  INTEGER,
    rotation               INTEGER,
    closed                 INTEGER,
    required_item_id       INTEGER,
    required_item_type_id  INTEGER,
    required_item_id2      INTEGER,
    required_item_type_id2 INTEGER,
    gold_amount            INTEGER,
    item_id                INTEGER,
    item_type_id           INTEGER,
    item_count             INTEGER,
    event_id               INTEGER,
    message_id             INTEGER,
    visibility             INTEGER
);