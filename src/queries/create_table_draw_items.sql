CREATE TABLE IF NOT EXISTS draw_items
(
    map_id    INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    x_coord   INTEGER,
    y_coord   INTEGER,
    item_id   INTEGER,
    item_type INTEGER,
    item_raw  INTEGER
)
