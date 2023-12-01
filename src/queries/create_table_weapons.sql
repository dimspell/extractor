-- DROP TABLE weapons;
CREATE TABLE IF NOT EXISTS weapons
(
    id            INTEGER,
    name          TEXT,
    description   TEXT,
    base_price    INTEGER,
    health_points INTEGER,
    magic_points  INTEGER,
    strength      INTEGER,
    agility       INTEGER,
    wisdom        INTEGER,
    tf            INTEGER,
    unk           INTEGER,
    trf           INTEGER,
    attack        INTEGER,
    defense       INTEGER,
    mag           INTEGER,
    durability    INTEGER,
    req_strength  INTEGER,
    req_zw        INTEGER,
    req_wisdom    INTEGER
)