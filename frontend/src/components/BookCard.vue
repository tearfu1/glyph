<script setup lang="ts">
import type { BookWithAuthor } from '@/types/book'

defineProps<{ book: BookWithAuthor }>()

const tagColors: Record<string, string> = {
  genre: 'bg-indigo-50 text-indigo-700',
  mood: 'bg-amber-50 text-amber-700',
  theme: 'bg-emerald-50 text-emerald-700',
  period: 'bg-slate-100 text-slate-600',
}
</script>

<template>
  <RouterLink
    :to="{ name: 'book', params: { id: book.id } }"
    class="group block bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden hover:shadow-md transition-shadow"
  >
    <div class="aspect-[2/3] bg-gray-100 overflow-hidden">
      <img
        v-if="book.cover_url"
        :src="book.cover_url"
        :alt="book.title"
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
      />
      <div v-else class="w-full h-full flex items-center justify-center text-gray-400">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-12 h-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
            d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
        </svg>
      </div>
    </div>
    <div class="p-3">
      <h3 class="font-semibold text-gray-900 text-sm leading-tight line-clamp-2 group-hover:text-indigo-600 transition-colors">
        {{ book.title }}
      </h3>
      <p class="mt-1 text-xs text-gray-500">{{ book.author_display_name }}</p>
      <p v-if="book.published_year" class="text-xs text-gray-400">{{ book.published_year }}</p>
      <div v-if="book.tags?.length" class="mt-2 flex flex-wrap gap-1">
        <span
          v-for="tag in book.tags"
          :key="tag.id"
          class="inline-block px-1.5 py-0.5 text-[10px] leading-tight font-medium rounded-md"
          :class="tagColors[tag.tag_type] || 'bg-gray-100 text-gray-600'"
        >
          {{ tag.name }}
        </span>
      </div>
    </div>
  </RouterLink>
</template>
