-- User Related Data
CREATE TABLE IF NOT EXISTS users
(
    id         INTEGER PRIMARY KEY NOT NULL,
    email      TEXT UNIQUE         NOT NULL,
    username   TEXT                NOT NULL,
    hash       TEXT                NOT NULL,
    created_at DATETIME            NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users_tokens
(
    id      INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER             NOT NULL,
    token   TEXT                NOT NULL,
    created DATETIME            NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires DATETIME            NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);