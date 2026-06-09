CREATE TABLE IF NOT EXISTS sprite_file_blobs (
    normalized_path TEXT PRIMARY KEY NOT NULL,
    data BLOB NOT NULL
);
