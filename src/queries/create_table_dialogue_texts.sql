CREATE TABLE IF NOT EXISTS dialogue_texts
(
    file_name    TEXT,
    id           INTEGER,
    text         TEXT,
    comment      TEXT,
    param1       INTEGER,
    param2       INTEGER,
    PRIMARY KEY (file_name, id)
)
