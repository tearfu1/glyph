import { ref, watch, type Ref } from 'vue'
import type { PaginatedResponse } from '@/types/book'

export function usePagination<T>(
  fetchFn: (page: number) => Promise<{ data: PaginatedResponse<T> }>,
) {
  const items = ref<T[]>([]) as Ref<T[]>
  const page = ref(1)
  const total = ref(0)
  const perPage = ref(20)
  const loading = ref(false)

  async function load() {
    loading.value = true
    try {
      const { data } = await fetchFn(page.value)
      items.value = data.data as T[]
      total.value = data.total
      perPage.value = data.per_page
    } catch {
      // silent
    } finally {
      loading.value = false
    }
  }

  function changePage(newPage: number) {
    page.value = newPage
  }

  watch(page, load)

  return { items, page, total, perPage, loading, load, changePage }
}
