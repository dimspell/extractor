CREATE TABLE IF NOT EXISTS map_sprites (
    map_id TEXT,
    sprite_id INTEGER,
    x INTEGER,
    y INTEGER,
    internal_sprite_id INTEGER,
    bottom_right_y INTEGER NOT NULL DEFAULT 0,
    bbox_left INTEGER,
    bbox_top INTEGER,
    bbox_right INTEGER,
    PRIMARY KEY (map_id, sprite_id)
);
CREATE INDEX IF NOT EXISTS idx_map_sprites_map_id ON map_sprites(map_id);
