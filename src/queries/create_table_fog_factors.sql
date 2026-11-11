CREATE TABLE IF NOT EXISTS fog_factors (
    level INTEGER NOT NULL,
    pair_index INTEGER NOT NULL,
    factor INTEGER NOT NULL,
    PRIMARY KEY (level, pair_index)
);
CREATE INDEX IF NOT EXISTS idx_fog_factors_level ON fog_factors(level);
