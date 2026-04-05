CREATE TABLE up_ai_answer (
    id          UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID    NOT NULL UNIQUE REFERENCES up_question (id) ON DELETE CASCADE,
    answer_text TEXT    NOT NULL,
    sources     JSONB   NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_up_ai_answer_question_id ON up_ai_answer (question_id);
