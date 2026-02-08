INSERT OR REPLACE INTO map_metadata (
    map_id,
    tiled_width,
    tiled_height,
    width_pixels,
    height_pixels,
    non_occluded_x,
    non_occluded_y,
    occluded_width,
    occluded_height
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
