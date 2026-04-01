CREATE TYPE tag_type AS ENUM ('genre', 'mood', 'theme', 'period');

CREATE TABLE up_tag (
    id       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name     VARCHAR(128) NOT NULL,
    tag_type tag_type    NOT NULL,
    UNIQUE (name, tag_type)
);

CREATE TABLE up_book_tag (
    book_id  UUID NOT NULL REFERENCES up_book (id) ON DELETE CASCADE,
    tag_id   UUID NOT NULL REFERENCES up_tag  (id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);
