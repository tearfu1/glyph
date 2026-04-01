import client from './client'
import type { GroupedTags } from '@/types/book'

export function getTags() {
  return client.get<GroupedTags[]>('/tags')
}
