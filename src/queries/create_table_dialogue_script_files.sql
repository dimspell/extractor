CREATE TABLE IF NOT EXISTS dialogue_script_files
(
    id             INTEGER PRIMARY KEY,
    file_path      TEXT NOT NULL,
    pgp_file_path  TEXT,
    map_name       TEXT
)
