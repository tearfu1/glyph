export interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
}

export interface BookWithAuthor {
  id: string
  title: string
  description: string | null
  cover_url: string | null
  isbn: string | null
  published_year: number | null
  author_id: string
  created_at: string
  updated_at: string
  author_login: string
  author_email: string
  author_display_name: string
  author_avatar_url: string | null
  tags: Tag[]
}

export type TagType = 'genre' | 'mood' | 'theme' | 'period'

export interface Tag {
  id: string
  name: string
  tag_type: TagType
}

export interface GroupedTags {
  tag_type: TagType
  tags: Tag[]
}
