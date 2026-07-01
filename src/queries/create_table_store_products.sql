CREATE TABLE IF NOT EXISTS store_products
(
    store_id     INTEGER REFERENCES stores(id) ON DELETE CASCADE,
    order_id     INTEGER,
    product_type INTEGER,
    product_id   INTEGER
)