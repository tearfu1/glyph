CREATE TABLE up_question (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id    UUID NOT NULL REFERENCES up_book (id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES up_user (id) ON DELETE CASCADE,
    text       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_up_question_book_id ON up_question (book_id);
CREATE INDEX idx_up_question_user_id ON up_question (user_id);

CREATE TABLE up_answer (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID NOT NULL UNIQUE REFERENCES up_question (id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES up_user     (id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE up_question_reaction (
    id          UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID    NOT NULL REFERENCES up_question (id) ON DELETE CASCADE,
    user_id     UUID    NOT NULL REFERENCES up_user     (id) ON DELETE CASCADE,
    is_like     BOOLEAN NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (question_id, user_id)
);

CREATE INDEX idx_up_question_reaction_question_id ON up_question_reaction (question_id);
