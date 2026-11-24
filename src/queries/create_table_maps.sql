CREATE TABLE IF NOT EXISTS maps
(
    id           INTEGER PRIMARY KEY,
    map_filename TEXT,
    map_name     TEXT,
    pgp_filename TEXT,
    dlg_filename TEXT,
    lighting     BOOLEAN
)