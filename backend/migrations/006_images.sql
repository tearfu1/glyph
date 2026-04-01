CREATE TABLE up_image (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID         NOT NULL REFERENCES up_user (id) ON DELETE CASCADE,
    url            TEXT         NOT NULL,
    thumbnail_url  TEXT,
    original_name  VARCHAR(255),
    size_bytes     INTEGER,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_up_image_user_id ON up_image (user_id);
