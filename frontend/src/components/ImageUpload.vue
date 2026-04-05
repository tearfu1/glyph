<script setup lang="ts">
import { ref, computed } from 'vue'
import { uploadImage } from '@/api/images'

const props = defineProps<{
  modelValue: string | null
  aspect?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [url: string]
}>()

const uploading = ref(false)
const error = ref('')
const dragging = ref(false)

const previewUrl = computed(() => props.modelValue || '')
const aspectClass = computed(() => {
  if (props.aspect === 'square') return 'aspect-square'
  if (props.aspect === 'book') return 'aspect-[2/3]'
  return 'aspect-square'
})

async function handleFile(file: File) {
  if (!file.type.startsWith('image/')) {
    error.value = 'Выберите изображение (JPEG, PNG, WebP, GIF)'
    return
  }
  if (file.size > 10 * 1024 * 1024) {
    error.value = 'Файл слишком большой (макс. 10 МБ)'
    return
  }

  error.value = ''
  uploading.value = true
  try {
    const { data } = await uploadImage(file)
    emit('update:modelValue', data.url)
  } catch {
    error.value = 'Не удалось загрузить изображение'
  } finally {
    uploading.value = false
  }
}

function onFileInput(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files?.[0]) {
    handleFile(input.files[0])
    input.value = ''
  }
}

function onDrop(e: DragEvent) {
  dragging.value = false
  if (e.dataTransfer?.files?.[0]) {
    handleFile(e.dataTransfer.files[0])
  }
}
</script>

<template>
  <div class="space-y-2">
    <div
      class="relative border-2 border-dashed rounded-xl overflow-hidden transition-colors cursor-pointer"
      :class="[
        aspectClass,
        dragging ? 'border-indigo-400 bg-indigo-50' : 'border-gray-300 hover:border-gray-400',
      ]"
      @dragover.prevent="dragging = true"
      @dragleave="dragging = false"
      @drop.prevent="onDrop"
      @click="($refs.fileInput as HTMLInputElement).click()"
    >
      <!-- Preview -->
      <img
        v-if="previewUrl && !uploading"
        :src="previewUrl"
        alt="Превью"
        class="w-full h-full object-cover"
      />

      <!-- Upload state -->
      <div v-else class="absolute inset-0 flex flex-col items-center justify-center text-gray-400">
        <div v-if="uploading" class="w-6 h-6 border-2 border-indigo-200 border-t-indigo-600 rounded-full animate-spin" />
        <template v-else>
          <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
          </svg>
          <span class="text-xs">Перетащите или нажмите</span>
        </template>
      </div>

      <input
        ref="fileInput"
        type="file"
        accept="image/jpeg,image/png,image/webp,image/gif"
        class="hidden"
        @change="onFileInput"
      />
    </div>

    <p v-if="error" class="text-xs text-red-500">{{ error }}</p>
  </div>
</template>
