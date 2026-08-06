CREATE TABLE IF NOT EXISTS map_inis
(
    id                      INTEGER,
    event_id_on_camera_move INTEGER REFERENCES events(event_id) ON DELETE SET NULL,
    start_pos_x             INTEGER,
    start_pos_y             INTEGER,
    map_id                  INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    monsters_filename       TEXT,
    npc_filename            TEXT,
    extra_filename          TEXT,
    cd_music_track_number   INTEGER
)