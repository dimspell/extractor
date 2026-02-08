DROP TABLE IF EXISTS monster_refs;
CREATE TABLE monster_refs
(
    file_path       TEXT,
    id              INTEGER,
    file_id         INTEGER,
    mon_id          INTEGER,
    pos_x           INTEGER,
    pos_y           INTEGER,
    loot1_item_id   INTEGER,
    loot1_item_type INTEGER,
    loot2_item_id   INTEGER,
    loot2_item_type INTEGER,
    loot3_item_id   INTEGER,
    loot3_item_type INTEGER
);