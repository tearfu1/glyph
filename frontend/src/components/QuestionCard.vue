<script setup lang="ts">
import type { QuestionWithUser } from '@/types/question'
import ReactionButtons from '@/components/ReactionButtons.vue'

defineProps<{
  question: QuestionWithUser
}>()

const emit = defineEmits<{
  react: [isLike: boolean]
}>()

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
}
</script>

<template>
  <div class="p-5 bg-white border border-gray-200 rounded-xl">
    <div class="flex items-start justify-between gap-4">
      <div class="flex items-center gap-3">
        <RouterLink :to="{ name: 'user', params: { id: question.user_id } }" class="shrink-0">
          <img
            v-if="question.user_avatar_url"
            :src="question.user_avatar_url"
            :alt="question.user_display_name"
            class="w-8 h-8 rounded-full object-cover"
          />
          <div
            v-else
            class="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-500 text-sm font-semibold"
          >
            {{ question.user_display_name?.charAt(0)?.toUpperCase() }}
          </div>
        </RouterLink>
        <RouterLink
          :to="{ name: 'user', params: { id: question.user_id } }"
          class="text-sm font-medium text-gray-900 hover:text-indigo-600 transition-colors"
        >
          {{ question.user_display_name }}
        </RouterLink>
        <span
          v-if="question.has_answer"
          class="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-emerald-50 text-emerald-700"
        >
          Есть ответ
        </span>
      </div>
      <span class="text-xs text-gray-400 shrink-0">{{ formatDate(question.created_at) }}</span>
    </div>
    <p class="mt-3 text-sm text-gray-700 leading-relaxed whitespace-pre-line">{{ question.text }}</p>

    <!-- Answer -->
    <div v-if="question.answer_text" class="mt-4 ml-4 pl-4 border-l-2 border-indigo-200">
      <div class="flex items-center gap-2 mb-1.5">
        <img
          v-if="question.answer_user_avatar_url"
          :src="question.answer_user_avatar_url"
          :alt="question.answer_user_display_name ?? ''"
          class="w-6 h-6 rounded-full object-cover"
        />
        <div
          v-else-if="question.answer_user_display_name"
          class="w-6 h-6 rounded-full bg-indigo-100 flex items-center justify-center text-indigo-600 text-[10px] font-semibold"
        >
          {{ question.answer_user_display_name.charAt(0).toUpperCase() }}
        </div>
        <span class="text-xs font-medium text-indigo-600">{{ question.answer_user_display_name }}</span>
        <span class="text-[10px] text-gray-400 uppercase tracking-wide font-semibold">Автор</span>
        <span v-if="question.answer_created_at" class="text-xs text-gray-400 ml-auto">{{ formatDate(question.answer_created_at) }}</span>
      </div>
      <p class="text-sm text-gray-700 leading-relaxed whitespace-pre-line">{{ question.answer_text }}</p>
    </div>

    <div class="mt-3">
      <ReactionButtons
        :like-count="question.like_count"
        :dislike-count="question.dislike_count"
        :user-reaction="question.user_reaction"
        @react="emit('react', $event)"
      />
    </div>
  </div>
</template>
