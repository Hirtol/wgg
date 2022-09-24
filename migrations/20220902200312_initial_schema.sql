-- Aggregate Ingredients
CREATE TABLE IF NOT EXISTS agg_ingredients
(
    id         INTEGER PRIMARY KEY NOT NULL,
    name       TEXT                NOT NULL,
    image_url  TEXT,
    created_by INTEGER             NOT NULL,
    created_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS agg_ingredients_links
(
    id               INTEGER PRIMARY KEY NOT NULL,
    aggregate_id     INTEGER             NOT NULL,
    provider_id      INTEGER             NOT NULL,
    -- Provider specific ingredient Id
    provider_ingr_id TEXT                NOT NULL,

    FOREIGN KEY (aggregate_id) REFERENCES agg_ingredients (id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES providers (id) ON DELETE CASCADE
);

---- Cart and order history ----

CREATE TABLE IF NOT EXISTS cart
(
    id           INTEGER PRIMARY KEY NOT NULL,
    user_id      INTEGER             NOT NULL,
    -- completed_at is filled in when an order is completed, therefore a current cart is null
    completed_at TIMESTAMP,
    -- picked_id is the current/final selected provider for a particular cart.
    picked_id    INTEGER,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (picked_id) REFERENCES providers (id) ON DELETE CASCADE
);

-- Ensure there is *always* a cart available to use.
CREATE TRIGGER IF NOT EXISTS cart_user_always_has_cart_insert AFTER INSERT
    ON cart
    WHEN (SELECT COUNT(*) FROM cart c WHERE c.user_id = new.user_id AND c.completed_at IS NULL) > 1
BEGIN
    SELECT RAISE(FAIL, 'At most one cart can be available at a time');
END;

CREATE TRIGGER IF NOT EXISTS cart_user_always_has_cart_update AFTER UPDATE
    ON cart
    WHEN NOT EXISTS(SELECT * FROM cart c WHERE c.user_id = old.user_id AND c.completed_at IS NULL)
BEGIN
    INSERT INTO cart(user_id) VALUES (old.user_id);
END;

CREATE TRIGGER IF NOT EXISTS cart_user_always_has_cart_delete AFTER DELETE
    ON cart
    WHEN NOT EXISTS(SELECT * FROM cart c WHERE c.user_id = old.user_id AND c.completed_at IS NULL)
BEGIN
    INSERT INTO cart(user_id) VALUES (old.user_id);
END;

CREATE TABLE IF NOT EXISTS cart_contents_notes
(
    id         INTEGER PRIMARY KEY NOT NULL,
    cart_id    INTEGER             NOT NULL,
    note       TEXT                NOT NULL,
    quantity   INTEGER             NOT NULL,
    created_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (cart_id) REFERENCES cart (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS cart_contents_provider
(
    id               INTEGER PRIMARY KEY NOT NULL,
    cart_id          INTEGER             NOT NULL,
    provider_id      INTEGER             NOT NULL,
    provider_product TEXT                NOT NULL,
    quantity         INTEGER             NOT NULL,
    created_at       TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(cart_id, provider_id, provider_product),
    FOREIGN KEY (cart_id) REFERENCES cart (id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES providers (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS cart_contents_aggregate
(
    id           INTEGER PRIMARY KEY NOT NULL,
    cart_id      INTEGER             NOT NULL,
    aggregate_id INTEGER             NOT NULL,
    quantity     INTEGER             NOT NULL,
    created_at   TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(cart_id, aggregate_id),
    FOREIGN KEY (cart_id) REFERENCES cart (id) ON DELETE CASCADE,
    FOREIGN KEY (aggregate_id) REFERENCES agg_ingredients (id) ON DELETE CASCADE
);

-- Running tally of current cart so that a user can at any time see the current prices.
-- Note that this *will* get out of date, which is why we need a final "Calculate" step for the true calculation.
CREATE TABLE IF NOT EXISTS cart_tally
(
    cart_id     INTEGER NOT NULL,
    provider_id INTEGER NOT NULL,
    price_cents INTEGER NOT NULL,

    PRIMARY KEY (cart_id, provider_id),
    FOREIGN KEY (cart_id) REFERENCES cart (id) ON DELETE CASCADE,
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
    created_at TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_admin   BOOLEAN             NOT NULL
);

-- Create a new cart as soon as a new user is created.
CREATE TRIGGER IF NOT EXISTS users_user_always_has_cart_insert AFTER INSERT
    ON users
BEGIN
    INSERT INTO cart(user_id) VALUES (new.id);
END;

CREATE TABLE IF NOT EXISTS users_tokens
(
    id      INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER             NOT NULL,
    token   TEXT UNIQUE         NOT NULL,
    created TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires TIMESTAMP           NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);