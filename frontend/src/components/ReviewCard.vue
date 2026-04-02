<script setup lang="ts">
import type { ReviewWithUser } from '@/types/review'
import StarRating from '@/components/StarRating.vue'
import ReactionButtons from '@/components/ReactionButtons.vue'

defineProps<{
  review: ReviewWithUser
  showBookLink?: boolean
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
        <RouterLink
          :to="{ name: 'user', params: { id: review.user_id } }"
          class="shrink-0"
        >
          <img
            v-if="review.user_avatar_url"
            :src="review.user_avatar_url"
            :alt="review.user_display_name"
            class="w-8 h-8 rounded-full object-cover"
          />
          <div
            v-else
            class="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-500 text-sm font-semibold"
          >
            {{ review.user_display_name?.charAt(0)?.toUpperCase() }}
          </div>
        </RouterLink>
        <div>
          <RouterLink
            :to="{ name: 'user', params: { id: review.user_id } }"
            class="text-sm font-medium text-gray-900 hover:text-indigo-600 transition-colors"
          >
            {{ review.user_display_name }}
          </RouterLink>
          <StarRating :model-value="review.rating" readonly size="sm" />
        </div>
      </div>
      <span class="text-xs text-gray-400 shrink-0">{{ formatDate(review.created_at) }}</span>
    </div>

    <p class="mt-3 text-sm text-gray-700 leading-relaxed whitespace-pre-line">{{ review.text }}</p>

    <div class="mt-3">
      <ReactionButtons
        :like-count="review.like_count"
        :dislike-count="review.dislike_count"
        :user-reaction="review.user_reaction"
        @react="emit('react', $event)"
      />
    </div>
  </div>
</template>
