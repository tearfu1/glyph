<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

const email = ref('')
const password = ref('')

async function onSubmit() {
  try {
    await auth.login({ email: email.value, password: password.value })
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch {
    // error is in auth.error
  }
}
</script>

<template>
  <div class="flex min-h-[calc(100vh-4rem)] items-center justify-center px-4">
    <div class="w-full max-w-md space-y-8">
      <div class="text-center">
        <h1 class="text-2xl font-bold text-gray-900">Вход в Glyph</h1>
        <p class="mt-2 text-sm text-gray-600">
          Нет аккаунта?
          <RouterLink to="/register" class="text-indigo-600 hover:text-indigo-500">
            Зарегистрируйтесь
          </RouterLink>
        </p>
      </div>

      <form @submit.prevent="onSubmit" class="space-y-6">
        <div v-if="auth.error" class="rounded-lg bg-red-50 p-4 text-sm text-red-700">
          {{ auth.error }}
        </div>

        <div>
          <label for="email" class="block text-sm font-medium text-gray-700">Email</label>
          <input
            id="email"
            v-model="email"
            type="email"
            required
            autocomplete="email"
            class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none sm:text-sm"
          />
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-gray-700">Пароль</label>
          <input
            id="password"
            v-model="password"
            type="password"
            required
            autocomplete="current-password"
            class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none sm:text-sm"
          />
        </div>

        <button
          type="submit"
          :disabled="auth.loading"
          class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ auth.loading ? 'Вход...' : 'Войти' }}
        </button>
      </form>
    </div>
  </div>
</template>
