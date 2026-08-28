CREATE TABLE IF NOT EXISTS monster_spawns
(
    serial          INTEGER NOT NULL,
    monster_no      INTEGER,
    x               INTEGER,
    y               INTEGER,
    persistent      INTEGER,
    burrow          INTEGER,
    item_group      INTEGER,
    drop_attribute  INTEGER
);
