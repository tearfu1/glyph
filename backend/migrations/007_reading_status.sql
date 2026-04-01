CREATE TYPE reading_status_type AS ENUM ('want_to_read', 'reading', 'read');

CREATE TABLE up_reading_status (
    id         UUID                PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID                NOT NULL REFERENCES up_user (id) ON DELETE CASCADE,
    book_id    UUID                NOT NULL REFERENCES up_book (id) ON DELETE CASCADE,
    status     reading_status_type NOT NULL,
    created_at TIMESTAMPTZ         NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ         NOT NULL DEFAULT now(),
    UNIQUE (user_id, book_id)
);

CREATE INDEX idx_up_reading_status_user_id ON up_reading_status (user_id);
CREATE INDEX idx_up_reading_status_book_id ON up_reading_status (book_id);
