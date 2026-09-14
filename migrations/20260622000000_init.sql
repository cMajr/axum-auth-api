CREATE TYPE user_role AS ENUM ('user', 'admin');

CREATE TABLE users (
    id          SERIAL PRIMARY KEY,
    username    VARCHAR(20)  NOT NULL UNIQUE,
    email       TEXT         NOT NULL UNIQUE,
    password    TEXT         NOT NULL,
    bio         TEXT,
    date_of_birth DATE,
    role        user_role    NOT NULL DEFAULT 'user',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
