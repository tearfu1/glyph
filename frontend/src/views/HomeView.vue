<script setup lang="ts">
import { ref, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import type { ReadingStatusType } from '@/types/reading-status'
import { getBooks } from '@/api/books'
import { getMyStatuses, setReadingStatus } from '@/api/reading-status'
import { usePagination } from '@/composables/usePagination'
import BookCard from '@/components/BookCard.vue'
import TagFilter from '@/components/TagFilter.vue'
import Pagination from '@/components/Pagination.vue'

const auth = useAuthStore()

const search = ref('')
const selectedTags = ref<string[]>([])

const {
  items: books,
  page,
  total,
  perPage,
  loading,
  load: fetchBooks,
} = usePagination((p) => getBooks({
  page: p,
  search: search.value || undefined,
  tags: selectedTags.value.length ? selectedTags.value : undefined,
}))

const statusMap = ref<Map<string, ReadingStatusType>>(new Map())

let searchTimeout: ReturnType<typeof setTimeout> | undefined

async function fetchStatuses() {
  if (!auth.isAuthenticated) return
  try {
    const { data } = await getMyStatuses()
    const map = new Map<string, ReadingStatusType>()
    for (const s of data) {
      map.set(s.book_id, s.status)
    }
    statusMap.value = map
  } catch {}
}

function getBookStatus(bookId: string): ReadingStatusType | null | undefined {
  if (!auth.isAuthenticated) return undefined
  return statusMap.value.get(bookId) ?? null
}

async function onStatusChange(bookId: string, status: ReadingStatusType) {
  statusMap.value.set(bookId, status)
  try {
    await setReadingStatus(bookId, status)
  } catch {
    statusMap.value.delete(bookId)
  }
}

function onSearchInput() {
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    page.value = 1
    fetchBooks()
  }, 300)
}

watch(selectedTags, () => {
  page.value = 1
  fetchBooks()
}, { deep: true })

fetchBooks()
fetchStatuses()
</script>

<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-8">Каталог книг</h1>

    <div class="flex flex-col lg:flex-row gap-8">
      <!-- Sidebar: search + tags -->
      <aside class="lg:w-64 shrink-0 space-y-6">
        <div>
          <input
            v-model="search"
            @input="onSearchInput"
            type="text"
            placeholder="Поиск по названию..."
            class="w-full px-4 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
          />
        </div>
        <TagFilter v-model="selectedTags" />
      </aside>

      <!-- Main content -->
      <div class="flex-1 min-w-0">
        <div v-if="loading" class="flex justify-center py-20">
          <div class="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin" />
        </div>

        <template v-else>
          <div v-if="books.length" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5 gap-4">
            <BookCard
              v-for="book in books"
              :key="book.id"
              :book="book"
              :reading-status="getBookStatus(book.id)"
              @update:reading-status="onStatusChange(book.id, $event)"
            />
          </div>

          <div v-else class="text-center py-20 text-gray-500">
            <p class="text-lg">Книги не найдены</p>
            <p class="text-sm mt-1">Попробуйте изменить параметры поиска</p>
          </div>

          <div class="mt-8">
            <Pagination v-model:page="page" :total="total" :per-page="perPage" />
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
