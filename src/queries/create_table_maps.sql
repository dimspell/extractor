-- DROP TABLE maps;
CREATE TABLE IF NOT EXISTS maps
(
    id           INTEGER,
    map_filename TEXT,
    map_name     TEXT,
    pgp_filename TEXT,
    dlg_filename TEXT,
    is_light     BOOLEAN
)