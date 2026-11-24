CREATE TABLE IF NOT EXISTS tile_grids
(
    id              INTEGER PRIMARY KEY,
    payload_length  INTEGER,
    grid_size       INTEGER,
    data_json       BLOB
);
