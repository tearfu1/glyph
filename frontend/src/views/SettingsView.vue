<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { getMySettings, updateMe } from '@/api/users'

const auth = useAuthStore()

const displayName = ref('')
const avatarUrl = ref('')
const loading = ref(true)
const saving = ref(false)
const success = ref(false)
const error = ref('')

async function fetchSettings() {
  loading.value = true
  try {
    const { data } = await getMySettings()
    displayName.value = data.display_name
    avatarUrl.value = data.avatar_url ?? ''
  } catch {
    error.value = 'Не удалось загрузить настройки'
  } finally {
    loading.value = false
  }
}

async function save() {
  if (!displayName.value.trim()) return
  saving.value = true
  success.value = false
  error.value = ''
  try {
    const { data } = await updateMe({
      display_name: displayName.value.trim(),
      avatar_url: avatarUrl.value.trim() || undefined,
    })
    // Update auth store so header reflects changes
    if (auth.user) {
      auth.user.display_name = data.display_name
      auth.user.avatar_url = data.avatar_url
    }
    success.value = true
  } catch {
    error.value = 'Не удалось сохранить изменения'
  } finally {
    saving.value = false
  }
}

onMounted(fetchSettings)
</script>

<template>
  <div class="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <RouterLink to="/" class="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-indigo-600 transition-colors mb-6">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
      </svg>
      Каталог
    </RouterLink>

    <h1 class="text-2xl font-bold text-gray-900 mb-8">Настройки профиля</h1>

    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-20">
      <div class="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin" />
    </div>

    <form v-else @submit.prevent="save" class="space-y-6">
      <!-- Avatar preview -->
      <div class="flex items-center gap-6">
        <img
          v-if="avatarUrl"
          :src="avatarUrl"
          alt="Аватар"
          class="w-16 h-16 rounded-full object-cover shadow-md"
        />
        <div
          v-else
          class="w-16 h-16 rounded-full bg-indigo-100 flex items-center justify-center text-indigo-600 text-xl font-bold shadow-md"
        >
          {{ displayName?.charAt(0)?.toUpperCase() }}
        </div>
        <div class="text-sm text-gray-500">
          <p class="font-medium text-gray-900">{{ auth.user?.login }}</p>
          <p>{{ auth.user?.email }}</p>
        </div>
      </div>

      <!-- Display name -->
      <div>
        <label for="displayName" class="block text-sm font-medium text-gray-700 mb-1">
          Отображаемое имя
        </label>
        <input
          id="displayName"
          v-model="displayName"
          type="text"
          required
          maxlength="128"
          class="w-full px-4 py-2.5 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
          placeholder="Ваше имя"
        />
      </div>

      <!-- Avatar URL -->
      <div>
        <label for="avatarUrl" class="block text-sm font-medium text-gray-700 mb-1">
          URL аватара
        </label>
        <input
          id="avatarUrl"
          v-model="avatarUrl"
          type="url"
          class="w-full px-4 py-2.5 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
          placeholder="https://example.com/avatar.jpg"
        />
        <p class="mt-1 text-xs text-gray-400">Прямая ссылка на изображение</p>
      </div>

      <!-- Messages -->
      <div v-if="success" class="p-3 bg-emerald-50 text-emerald-700 text-sm rounded-lg">
        Изменения сохранены
      </div>
      <div v-if="error" class="p-3 bg-red-50 text-red-700 text-sm rounded-lg">
        {{ error }}
      </div>

      <!-- Submit -->
      <button
        type="submit"
        :disabled="saving || !displayName.trim()"
        class="px-6 py-2.5 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {{ saving ? 'Сохранение...' : 'Сохранить' }}
      </button>
    </form>
  </div>
</template>
