CREATE TABLE IF NOT EXISTS map_info_multi
(
    id                      INTEGER PRIMARY KEY,
    self_name               TEXT,
    anchor_tile_index       INTEGER,
    stride                  INTEGER,
    row_step                INTEGER,
    start_pos_a_x           INTEGER,
    start_pos_a_y           INTEGER,
    start_pos_b_x           INTEGER,
    start_pos_b_y           INTEGER,
    transition_target       TEXT,
    mon_filename            TEXT,
    npc_spawns_json         BLOB
);
