import client from './client'
import type { BookWithAuthor, PaginatedResponse } from '@/types/book'

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
