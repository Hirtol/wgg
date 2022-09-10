-- Aggregate Ingredients
CREATE TABLE IF NOT EXISTS agg_ingredients
(
    id         INTEGER PRIMARY KEY NOT NULL,
    name       TEXT                NOT NULL,
    created_by INTEGER             NOT NULL,
    created_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

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

-- User Related Data
CREATE TABLE IF NOT EXISTS users
(
    id         INTEGER PRIMARY KEY NOT NULL,
    email      TEXT UNIQUE         NOT NULL,
    username   TEXT                NOT NULL,
    hash       TEXT                NOT NULL,
    created_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users_tokens
(
    id      INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER             NOT NULL,
    token   TEXT UNIQUE         NOT NULL,
    created TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires TIMESTAMP           NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);