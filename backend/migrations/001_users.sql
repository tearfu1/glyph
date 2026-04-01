CREATE TYPE user_role AS ENUM ('user', 'premium', 'author', 'admin');

CREATE TABLE up_user (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    login       VARCHAR(64)  NOT NULL UNIQUE,
    email       VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    display_name  VARCHAR(128) NOT NULL,
    role        user_role    NOT NULL DEFAULT 'user',
    avatar_url  TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_up_user_email ON up_user (email);
CREATE INDEX idx_up_user_login ON up_user (login);
