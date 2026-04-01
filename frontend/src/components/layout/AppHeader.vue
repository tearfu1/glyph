<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
</script>

<template>
  <header class="bg-white border-b border-gray-200">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center justify-between h-16">
        <RouterLink to="/" class="text-xl font-bold text-indigo-600">
          Glyph
        </RouterLink>

        <nav class="flex items-center gap-4">
          <template v-if="auth.isAuthenticated">
            <span class="text-sm text-gray-600">
              {{ auth.user?.display_name }}
            </span>
            <button
              @click="auth.logout()"
              class="text-sm text-gray-500 hover:text-gray-700 cursor-pointer"
            >
              Выйти
            </button>
          </template>
          <template v-else>
            <RouterLink
              to="/login"
              class="text-sm text-gray-600 hover:text-gray-900"
            >
              Войти
            </RouterLink>
            <RouterLink
              to="/register"
              class="inline-flex items-center px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-lg hover:bg-indigo-700"
            >
              Регистрация
            </RouterLink>
          </template>
        </nav>
      </div>
    </div>
  </header>
</template>
