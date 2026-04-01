<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { GroupedTags, Tag } from '@/types/book'
import { getTags } from '@/api/tags'

const props = defineProps<{ modelValue: string[] }>()
const emit = defineEmits<{ 'update:modelValue': [value: string[]] }>()

const groups = ref<GroupedTags[]>([])
const loading = ref(true)

const typeLabels: Record<string, string> = {
  genre: 'Жанр',
  mood: 'Настроение',
  theme: 'Тема',
  period: 'Эпоха',
}

onMounted(async () => {
  try {
    const { data } = await getTags()
    groups.value = data
  } finally {
    loading.value = false
  }
})

function toggle(tag: Tag) {
  const selected = [...props.modelValue]
  const idx = selected.indexOf(tag.id)
  if (idx >= 0) {
    selected.splice(idx, 1)
  } else {
    selected.push(tag.id)
  }
  emit('update:modelValue', selected)
}

function isSelected(tag: Tag) {
  return props.modelValue.includes(tag.id)
}
</script>

<template>
  <div v-if="!loading && groups.length" class="space-y-4">
    <div v-for="group in groups" :key="group.tag_type">
      <h4 class="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
        {{ typeLabels[group.tag_type] || group.tag_type }}
      </h4>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="tag in group.tags"
          :key="tag.id"
          @click="toggle(tag)"
          class="px-3 py-1 text-sm rounded-full border transition-colors cursor-pointer"
          :class="isSelected(tag)
            ? 'bg-indigo-600 text-white border-indigo-600'
            : 'bg-white text-gray-700 border-gray-300 hover:border-indigo-400'"
        >
          {{ tag.name }}
        </button>
      </div>
    </div>
  </div>
</template>
