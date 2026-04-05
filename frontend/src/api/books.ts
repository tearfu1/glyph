import client from './client'
import type { BookWithAuthor, PaginatedResponse } from '@/types/book'
import type { ReadingStatusType } from '@/types/reading-status'

export interface BookQueryParams {
  page?: number
  search?: string
  tags?: string[]
}

export function getBooks(params: BookQueryParams = {}) {
  return client.get<PaginatedResponse<BookWithAuthor>>('/books', {
    params: {
      page: params.page,
      search: params.search || undefined,
      tags: params.tags?.length ? params.tags.join(',') : undefined,
    },
  })
}

export function getBook(id: string) {
  return client.get<BookWithAuthor>(`/books/${id}`)
}

export function updateBook(id: string, data: { cover_url?: string; title?: string; description?: string }) {
  return client.put<BookWithAuthor>(`/books/${id}`, data)
}

export function getShelfBooks(userId: string, status?: ReadingStatusType, page = 1) {
  return client.get<PaginatedResponse<BookWithAuthor>>(`/books/shelf/${userId}`, {
    params: { status, page },
  })
}
