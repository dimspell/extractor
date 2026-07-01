CREATE TABLE IF NOT EXISTS dialogue_paragraphs
(
    file_name         TEXT,
    id                INTEGER,
    text              TEXT,
    comment           TEXT,
    param1            INTEGER,
    wave_ini_entry_id INTEGER REFERENCES wave_inis(id) ON DELETE SET NULL,
    PRIMARY KEY (file_name, id)
)
