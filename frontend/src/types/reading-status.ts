export type ReadingStatusType = 'want_to_read' | 'reading' | 'read'

export interface ReadingStatus {
  id: string
  user_id: string
  book_id: string
  status: ReadingStatusType
  created_at: string
  updated_at: string
}
