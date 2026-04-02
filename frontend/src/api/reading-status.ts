import client from './client'
import type { ReadingStatus, ReadingStatusType } from '@/types/reading-status'

export function getMyStatuses() {
  return client.get<ReadingStatus[]>('/reading-statuses/my')
}

export function setReadingStatus(bookId: string, status: ReadingStatusType) {
  return client.post<ReadingStatus>(`/books/${bookId}/reading-status`, { status })
}

export function deleteReadingStatus(bookId: string) {
  return client.delete(`/books/${bookId}/reading-status`)
}
