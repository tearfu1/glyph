export interface Question {
  id: string
  book_id: string
  user_id: string
  text: string
  created_at: string
  updated_at: string
}

export interface QuestionWithUser {
  id: string
  book_id: string
  user_id: string
  text: string
  created_at: string
  updated_at: string
  user_login: string
  user_display_name: string
  user_avatar_url: string | null
  like_count: number
  dislike_count: number
  user_reaction: boolean | null
  has_answer: boolean
  answer_text: string | null
  answer_created_at: string | null
  answer_user_display_name: string | null
  answer_user_avatar_url: string | null
}
