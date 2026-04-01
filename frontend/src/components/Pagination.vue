<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  page: number
  total: number
  perPage: number
}>()

const emit = defineEmits<{ 'update:page': [value: number] }>()

const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.perPage)))

const pages = computed(() => {
  const current = props.page
  const last = totalPages.value
  const range: number[] = []

  const start = Math.max(1, current - 2)
  const end = Math.min(last, current + 2)

  if (start > 1) {
    range.push(1)
    if (start > 2) range.push(-1) // ellipsis
  }

  for (let i = start; i <= end; i++) {
    range.push(i)
  }

  if (end < last) {
    if (end < last - 1) range.push(-1) // ellipsis
    range.push(last)
  }

  return range
})
</script>

<template>
  <nav v-if="totalPages > 1" class="flex items-center justify-center gap-1">
    <button
      :disabled="page <= 1"
      @click="emit('update:page', page - 1)"
      class="px-3 py-2 text-sm rounded-lg border border-gray-300 hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
    >
      &larr;
    </button>

    <template v-for="(p, i) in pages" :key="i">
      <span v-if="p === -1" class="px-2 text-gray-400">...</span>
      <button
        v-else
        @click="emit('update:page', p)"
        class="px-3 py-2 text-sm rounded-lg border cursor-pointer"
        :class="p === page
          ? 'bg-indigo-600 text-white border-indigo-600'
          : 'border-gray-300 hover:bg-gray-50'"
      >
        {{ p }}
      </button>
    </template>

    <button
      :disabled="page >= totalPages"
      @click="emit('update:page', page + 1)"
      class="px-3 py-2 text-sm rounded-lg border border-gray-300 hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
    >
      &rarr;
    </button>
  </nav>
</template>
