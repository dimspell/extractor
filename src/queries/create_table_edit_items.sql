CREATE TABLE IF NOT EXISTS edit_items
(
    id                    INTEGER,
    name                  TEXT,
    description           TEXT,
    base_price            INTEGER,
    padding1              INTEGER,
    health_points         INTEGER,
    mana_points           INTEGER,
    strength              INTEGER,
    agility               INTEGER,
    wisdom                INTEGER,
    constitution          INTEGER,
    to_dodge              INTEGER,
    to_hit                INTEGER,
    offense               INTEGER,
    defense               INTEGER,
    padding2              INTEGER,
    item_destroying_power INTEGER,
    padding3              INTEGER,
    modifies_item         BOOLEAN,
    additional_effect     INTEGER
)
