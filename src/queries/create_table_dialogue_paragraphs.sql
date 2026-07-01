CREATE TABLE IF NOT EXISTS dialogue_paragraphs
(
    file_id              INTEGER NOT NULL REFERENCES dialogue_script_files(id),
    id                   INTEGER,
    text                 TEXT,
    comment              TEXT,
    param1               INTEGER,
    wave_ini_entry_id    INTEGER REFERENCES wave_inis(id) ON DELETE SET NULL,
    PRIMARY KEY (file_id, id)
)
