import client from './client'
import type { PaginatedResponse } from '@/types/book'
import type { Question, QuestionWithUser } from '@/types/question'

export function getQuestions(bookId: string, page = 1) {
  return client.get<PaginatedResponse<QuestionWithUser>>(`/books/${bookId}/questions`, {
    params: { page },
  })
}

export function createQuestion(bookId: string, text: string) {
  return client.post<Question>(`/books/${bookId}/questions`, { text })
}

export function addReaction(questionId: string, isLike: boolean) {
  return client.post(`/questions/${questionId}/reaction`, { is_like: isLike })
}

export function removeReaction(questionId: string) {
  return client.delete(`/questions/${questionId}/reaction`)
}
