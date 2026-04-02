import client from './client'
import type { PaginatedResponse } from '@/types/book'
import type { Review, ReviewWithUser } from '@/types/review'

export function getReviews(bookId: string, page = 1) {
  return client.get<PaginatedResponse<ReviewWithUser>>(`/books/${bookId}/reviews`, {
    params: { page },
  })
}

export function getMyReview(bookId: string) {
  return client.get<ReviewWithUser | null>(`/books/${bookId}/reviews/my`)
}

export function createReview(bookId: string, data: { rating: number; text: string }) {
  return client.post<Review>(`/books/${bookId}/reviews`, data)
}

export function updateReview(reviewId: string, data: { rating: number; text: string }) {
  return client.put<Review>(`/reviews/${reviewId}`, data)
}

export function deleteReview(reviewId: string) {
  return client.delete(`/reviews/${reviewId}`)
}

export function addReaction(reviewId: string, isLike: boolean) {
  return client.post(`/reviews/${reviewId}/reaction`, { is_like: isLike })
}

export function removeReaction(reviewId: string) {
  return client.delete(`/reviews/${reviewId}/reaction`)
}
