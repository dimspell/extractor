CREATE TABLE IF NOT EXISTS party_refs
(
    id                    INTEGER,
    full_name             TEXT,
    job_name              TEXT,
    root_map_id           INTEGER,
    npc_id                INTEGER,
    dlg_when_not_in_party INTEGER,
    dlg_when_in_party     INTEGER,
    ghost_face_id         INTEGER
)