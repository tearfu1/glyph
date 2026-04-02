<script setup lang="ts">
defineProps<{
  likeCount: number
  dislikeCount: number
  userReaction: boolean | null
}>()

const emit = defineEmits<{
  react: [isLike: boolean]
}>()
</script>

<template>
  <div class="flex items-center gap-4">
    <button
      @click="emit('react', true)"
      class="inline-flex items-center gap-1.5 text-sm transition-colors cursor-pointer"
      :class="userReaction === true ? 'text-indigo-600' : 'text-gray-400 hover:text-indigo-600'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M14 10h4.764a2 2 0 011.789 2.894l-3.5 7A2 2 0 0115.263 21h-4.017c-.163 0-.326-.02-.485-.06L7 20m7-10V5a2 2 0 00-2-2h-.095c-.5 0-.905.405-.905.905 0 .714-.211 1.412-.608 2.006L7 11v9m7-10h-2M7 20H5a2 2 0 01-2-2v-6a2 2 0 012-2h2.5" />
      </svg>
      <span v-if="likeCount">{{ likeCount }}</span>
    </button>
    <button
      @click="emit('react', false)"
      class="inline-flex items-center gap-1.5 text-sm transition-colors cursor-pointer"
      :class="userReaction === false ? 'text-red-500' : 'text-gray-400 hover:text-red-500'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M10 14H5.236a2 2 0 01-1.789-2.894l3.5-7A2 2 0 018.736 3h4.018c.163 0 .326.02.485.06L17 4m-7 10v2a3.5 3.5 0 003.5 3.5h.095c.5 0 .905-.405.905-.905 0-.714.211-1.412.608-2.006L17 13V4m-7 10h2m5-6h2a2 2 0 012 2v6a2 2 0 01-2 2h-2.5" />
      </svg>
      <span v-if="dislikeCount">{{ dislikeCount }}</span>
    </button>
  </div>
</template>
