CREATE TABLE IF NOT EXISTS monster_inis
(
    id              INTEGER PRIMARY KEY,
    name            TEXT,
    sprite_filename TEXT,
    attack          INTEGER,
    hit             INTEGER,
    death           INTEGER,
    walking         INTEGER,
    casting_magic   INTEGER
)