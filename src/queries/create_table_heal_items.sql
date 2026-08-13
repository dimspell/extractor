CREATE TABLE IF NOT EXISTS heal_items
(
    id             INTEGER,
    name           TEXT,
    description    TEXT,
    base_price     INTEGER,
    runtime_item_index_slot INTEGER,
    health_points  INTEGER,
    mana_points    INTEGER,
    restores_full_health BOOLEAN,
    restores_full_mana BOOLEAN,
    cures_poison    BOOLEAN,
    cures_petrification BOOLEAN,
    cures_polymorph BOOLEAN,
    reserved_trailer BLOB
);
