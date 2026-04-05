export interface AiAnswerSource {
  text: string
  score: number
  book: string
}

export interface AiAnswer {
  id: string
  question_id: string
  answer_text: string
  sources: AiAnswerSource[]
  created_at: string
}
