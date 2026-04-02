import client from './client'
import type { PublicUser } from '@/types/user'
import type { PaginatedResponse } from '@/types/book'
import type { ReviewWithUser } from '@/types/review'

export function getUserProfile(userId: string) {
  return client.get<PublicUser>(`/users/${userId}/profile`)
}

export function getUserReviews(userId: string, page = 1) {
  return client.get<PaginatedResponse<ReviewWithUser>>(`/users/${userId}/reviews`, {
    params: { page },
  })
}

export function getMySettings() {
  return client.get<PublicUser>('/users/me/settings')
}

export function updateMe(data: { display_name?: string; avatar_url?: string }) {
  return client.put<PublicUser>('/users/me', data)
}
