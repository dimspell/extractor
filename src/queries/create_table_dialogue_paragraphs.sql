CREATE TABLE IF NOT EXISTS dialogue_paragraphs
(
    file_id              INTEGER NOT NULL,
    id                   INTEGER,
    text                 TEXT,
    comment              TEXT,
    param1               INTEGER,
    wave_ini_entry_id    INTEGER,
    PRIMARY KEY (file_id, id)
)
