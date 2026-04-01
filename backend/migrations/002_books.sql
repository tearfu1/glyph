CREATE TABLE up_book (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    title           VARCHAR(512) NOT NULL,
    description     TEXT,
    cover_url       TEXT,
    isbn            VARCHAR(20),
    published_year  SMALLINT,
    author_id       UUID         NOT NULL REFERENCES up_user (id) ON DELETE CASCADE,
    search_vector   TSVECTOR,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_up_book_author_id ON up_book (author_id);
CREATE INDEX idx_up_book_search_vector ON up_book USING GIN (search_vector);

CREATE OR REPLACE FUNCTION up_book_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector := to_tsvector('simple', coalesce(NEW.title, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_up_book_search_vector
    BEFORE INSERT OR UPDATE OF title
    ON up_book
    FOR EACH ROW EXECUTE FUNCTION up_book_search_vector_update();
