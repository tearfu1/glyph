export interface Review {
  id: string
  book_id: string
  user_id: string
  rating: number
  text: string
  created_at: string
  updated_at: string
}

export interface ReviewWithUser {
  id: string
  book_id: string
  user_id: string
  rating: number
  text: string
  created_at: string
  updated_at: string
  user_login: string
  user_display_name: string
  user_avatar_url: string | null
  like_count: number
  dislike_count: number
  user_reaction: boolean | null
}
