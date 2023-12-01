CREATE TABLE IF NOT EXISTS monster_refs
(
    id              INTEGER,
    file_id         INTEGER,
    mon_id          INTEGER,
    pos_x           INTEGER,
    pos_y           INTEGER,
    loot1_item_id   BOOLEAN,
    loot1_item_type BOOLEAN,
    loot2_item_id   BOOLEAN,
    loot2_item_type BOOLEAN,
    loot3_item_id   BOOLEAN,
    loot3_item_type BOOLEAN
)