CREATE TABLE up_review (
    id         UUID      PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id    UUID      NOT NULL REFERENCES up_book (id) ON DELETE CASCADE,
    user_id    UUID      NOT NULL REFERENCES up_user (id) ON DELETE CASCADE,
    rating     SMALLINT  NOT NULL CHECK (rating BETWEEN 1 AND 5),
    text       TEXT      NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (book_id, user_id)
);

CREATE INDEX idx_up_review_book_id ON up_review (book_id);
CREATE INDEX idx_up_review_user_id ON up_review (user_id);

CREATE TABLE up_review_reaction (
    id         UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id  UUID    NOT NULL REFERENCES up_review (id) ON DELETE CASCADE,
    user_id    UUID    NOT NULL REFERENCES up_user   (id) ON DELETE CASCADE,
    is_like    BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (review_id, user_id)
);

CREATE INDEX idx_up_review_reaction_review_id ON up_review_reaction (review_id);
