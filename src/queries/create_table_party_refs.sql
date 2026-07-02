CREATE TABLE IF NOT EXISTS party_refs
(
    id                    INTEGER,
    full_name             TEXT,
    job_name              TEXT,
    root_map_id           INTEGER NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    npc_id                INTEGER NOT NULL REFERENCES npc_inis(id) ON DELETE CASCADE,
    party_dlg_file_id     INTEGER NOT NULL REFERENCES dialogue_script_files(id),
    dlg_when_not_in_party INTEGER,
    dlg_when_in_party     INTEGER,
    ghost_face_id         INTEGER,
    FOREIGN KEY (party_dlg_file_id, dlg_when_not_in_party) REFERENCES dialogue_scripts(dialog_file_id, id),
    FOREIGN KEY (party_dlg_file_id, dlg_when_in_party) REFERENCES dialogue_scripts(dialog_file_id, id)
)