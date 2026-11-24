CREATE TABLE IF NOT EXISTS stores
(
    id                  INTEGER PRIMARY KEY,
    store_name          TEXT,
    inn_night_cost      INTEGER,
    price_modifier INTEGER,
    invitation          TEXT,
    haggle_success      TEXT,
    haggle_fail         TEXT
)
