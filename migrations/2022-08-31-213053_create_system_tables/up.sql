-- Aggregate Ingredients
CREATE TABLE IF NOT EXISTS agg_ingredients
(
    id         INTEGER PRIMARY KEY NOT NULL,
    name       TEXT                NOT NULL,
    created_by INTEGER             NOT NULL,
    created_at DATETIME            NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS agg_ingredients_links
(
    id               INTEGER NOT NULL,
    provider_id      INTEGER NOT NULL,
    -- Provider specific ingredient Id
    provider_ingr_id TEXT    NOT NULL,

    PRIMARY KEY (id, provider_id),
    FOREIGN KEY (id) REFERENCES agg_ingredients (id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES providers (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS providers
(
    id   INTEGER PRIMARY KEY NOT NULL,
    name TEXT                NOT NULL
);