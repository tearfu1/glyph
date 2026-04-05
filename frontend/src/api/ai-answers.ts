import client from './client'
import type { AiAnswer } from '@/types/ai-answer'

export function getAiAnswer(questionId: string) {
  return client.get<AiAnswer>(`/questions/${questionId}/ai-answer`)
}

export function generateAiAnswer(questionId: string) {
  return client.post<AiAnswer>(`/questions/${questionId}/ai-answer`)
}
