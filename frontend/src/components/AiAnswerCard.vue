<script setup lang="ts">
import { ref } from 'vue'
import type { AiAnswer } from '@/types/ai-answer'

defineProps<{
  answer: AiAnswer | null
  loading: boolean
  canGenerate: boolean
}>()

const emit = defineEmits<{
  generate: []
}>()

const sourcesOpen = ref(false)

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
}

function formatScore(score: number): string {
  return (score * 100).toFixed(0) + '%'
}
</script>

<template>
  <!-- Loading state -->
  <div v-if="loading" class="mt-4 ml-4 pl-4 border-l-2 border-violet-200">
    <div class="flex items-center gap-2 mb-2">
      <div class="w-4 h-4 border-2 border-violet-300 border-t-violet-600 rounded-full animate-spin" />
      <span class="text-xs text-violet-600 font-medium">Генерирую AI-ответ...</span>
    </div>
  </div>

  <!-- AI answer -->
  <div v-else-if="answer" class="mt-4 ml-4 pl-4 border-l-2 border-violet-200">
    <div class="flex items-center gap-2 mb-1.5">
      <span class="inline-flex items-center gap-1 px-2 py-0.5 text-[10px] font-semibold rounded-full bg-violet-100 text-violet-700 uppercase tracking-wide">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2a1 1 0 01.894.553l2.382 4.764 5.27.766a1 1 0 01.554 1.706l-3.813 3.717.9 5.249a1 1 0 01-1.451 1.054L12 17.347l-4.736 2.462a1 1 0 01-1.451-1.054l.9-5.249L2.9 9.789a1 1 0 01.554-1.706l5.27-.766L11.106 2.553A1 1 0 0112 2z" />
        </svg>
        AI-ответ
      </span>
      <span class="text-xs text-gray-400 ml-auto">{{ formatDate(answer.created_at) }}</span>
    </div>

    <p class="text-sm text-gray-700 leading-relaxed whitespace-pre-line bg-violet-50/40 rounded-lg px-3 py-2.5">
      {{ answer.answer_text }}
    </p>

    <!-- Sources -->
    <div v-if="answer.sources.length" class="mt-2">
      <button
        @click="sourcesOpen = !sourcesOpen"
        class="flex items-center gap-1 text-xs text-violet-600 hover:text-violet-800 transition-colors cursor-pointer"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-3.5 h-3.5 transition-transform"
          :class="sourcesOpen ? 'rotate-90' : ''"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
        </svg>
        {{ sourcesOpen ? 'Скрыть источники' : `Источники (${answer.sources.length})` }}
      </button>

      <div v-if="sourcesOpen" class="mt-2 space-y-2">
        <div
          v-for="(source, i) in answer.sources"
          :key="i"
          class="px-3 py-2 bg-white border border-violet-100 rounded-lg text-xs text-gray-600"
        >
          <div class="flex items-center justify-between gap-2 mb-1">
            <span class="font-medium text-gray-700 truncate">{{ source.book }}</span>
            <span class="shrink-0 px-1.5 py-0.5 rounded bg-violet-100 text-violet-700 font-semibold">
              {{ formatScore(source.score) }}
            </span>
          </div>
          <p class="leading-relaxed line-clamp-3 text-gray-500">{{ source.text }}</p>
        </div>
      </div>
    </div>
  </div>

  <!-- Generate button -->
  <div v-else-if="canGenerate" class="mt-3 ml-4">
    <button
      @click="emit('generate')"
      class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-violet-700 bg-violet-50 border border-violet-200 rounded-lg hover:bg-violet-100 transition-colors cursor-pointer"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2a1 1 0 01.894.553l2.382 4.764 5.27.766a1 1 0 01.554 1.706l-3.813 3.717.9 5.249a1 1 0 01-1.451 1.054L12 17.347l-4.736 2.462a1 1 0 01-1.451-1.054l.9-5.249L2.9 9.789a1 1 0 01.554-1.706l5.27-.766L11.106 2.553A1 1 0 0112 2z" />
      </svg>
      Сгенерировать AI-ответ
    </button>
  </div>
</template>
