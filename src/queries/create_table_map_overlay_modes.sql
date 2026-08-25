CREATE TABLE IF NOT EXISTS map_overlay_modes (
    map_id TEXT NOT NULL,
    overlay_index INTEGER NOT NULL,
    mode INTEGER NOT NULL,
    transparency_mode INTEGER NOT NULL,
    draw_enable INTEGER NOT NULL,
    PRIMARY KEY (map_id, overlay_index)
);
