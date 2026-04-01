CREATE TABLE IF NOT EXISTS heal_items
(
    id             INTEGER,
    name           TEXT,
    description    TEXT,
    base_price     INTEGER,
    padding1       INTEGER,
    padding2       INTEGER,
    padding3       INTEGER,
    health_points  INTEGER,
    mana_points    INTEGER,
    restore_full_health BOOLEAN,
    restore_full_mana BOOLEAN,
    poison_heal    BOOLEAN,
    petrif_heal    BOOLEAN,
    polimorph_heal BOOLEAN,
    padding4       INTEGER,
    padding5       INTEGER
)
