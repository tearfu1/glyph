import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { setAccessToken } from '@/api/client'
import * as authApi from '@/api/auth'
import type { PublicUser, LoginRequest, RegisterRequest } from '@/types/user'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<PublicUser | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const isAuthenticated = computed(() => !!user.value)
  const userRole = computed(() => user.value?.role ?? null)

  function setAuth(accessToken: string, publicUser: PublicUser) {
    setAccessToken(accessToken)
    user.value = publicUser
  }

  function clearAuth() {
    setAccessToken(null)
    user.value = null
  }

  async function login(data: LoginRequest) {
    loading.value = true
    error.value = null
    try {
      const res = await authApi.login(data)
      setAuth(res.data.access_token, res.data.user)
    } catch (e: any) {
      error.value = e.response?.data?.error ?? 'Ошибка входа'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function register(data: RegisterRequest) {
    loading.value = true
    error.value = null
    try {
      const res = await authApi.register(data)
      setAuth(res.data.access_token, res.data.user)
    } catch (e: any) {
      error.value = e.response?.data?.error ?? 'Ошибка регистрации'
      throw e
    } finally {
      loading.value = false
    }
  }

  async function tryRefresh() {
    try {
      const res = await authApi.refresh()
      setAuth(res.data.access_token, res.data.user)
    } catch {
      clearAuth()
    }
  }

  async function logout() {
    try {
      await authApi.logout()
    } finally {
      clearAuth()
    }
  }

  return {
    user,
    loading,
    error,
    isAuthenticated,
    userRole,
    login,
    register,
    tryRefresh,
    logout,
  }
})
