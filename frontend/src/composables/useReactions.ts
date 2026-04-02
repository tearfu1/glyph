import { useAuthStore } from '@/stores/auth'

interface Reactable {
  id: string
  user_reaction: boolean | null
}

export function useReactions(
  addReaction: (id: string, isLike: boolean) => Promise<unknown>,
  removeReaction: (id: string) => Promise<unknown>,
  onUpdate: () => Promise<void>,
) {
  const auth = useAuthStore()

  async function toggleReaction(item: Reactable, isLike: boolean) {
    if (!auth.isAuthenticated) return
    try {
      if (item.user_reaction === isLike) {
        await removeReaction(item.id)
      } else {
        await addReaction(item.id, isLike)
      }
      await onUpdate()
    } catch {}
  }

  return { toggleReaction }
}
