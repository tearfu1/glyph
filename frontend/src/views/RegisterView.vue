<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const login = ref('')
const email = ref('')
const password = ref('')
const displayName = ref('')

async function onSubmit() {
  try {
    await auth.register({
      login: login.value,
      email: email.value,
      password: password.value,
      display_name: displayName.value,
    })
    router.push('/')
  } catch {
    // error is in auth.error
  }
}
</script>

<template>
  <div class="flex min-h-[calc(100vh-4rem)] items-center justify-center px-4">
    <div class="w-full max-w-md space-y-8">
      <div class="text-center">
        <h1 class="text-2xl font-bold text-gray-900">Регистрация</h1>
        <p class="mt-2 text-sm text-gray-600">
          Уже есть аккаунт?
          <RouterLink to="/login" class="text-indigo-600 hover:text-indigo-500">
            Войдите
          </RouterLink>
        </p>
      </div>

      <form @submit.prevent="onSubmit" class="space-y-6">
        <div v-if="auth.error" class="rounded-lg bg-red-50 p-4 text-sm text-red-700">
          {{ auth.error }}
        </div>

        <div>
          <label for="login" class="block text-sm font-medium text-gray-700">Логин</label>
          <input
            id="login"
            v-model="login"
            type="text"
            required
            minlength="3"
            maxlength="64"
            autocomplete="username"
            class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none sm:text-sm"
          />
        </div>

        <div>
          <label for="display-name" class="block text-sm font-medium text-gray-700">Отображаемое имя</label>
          <input
            id="display-name"
            v-model="displayName"
            type="text"
            required
            maxlength="128"
            class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none sm:text-sm"
          />
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
            minlength="8"
            autocomplete="new-password"
            class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 focus:outline-none sm:text-sm"
          />
          <p class="mt-1 text-xs text-gray-500">Минимум 8 символов</p>
        </div>

        <button
          type="submit"
          :disabled="auth.loading"
          class="w-full rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ auth.loading ? 'Регистрация...' : 'Зарегистрироваться' }}
        </button>
      </form>
    </div>
  </div>
</template>
