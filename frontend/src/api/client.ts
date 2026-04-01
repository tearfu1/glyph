import axios from 'axios'
import router from '@/router'

const client = axios.create({
  baseURL: '/api',
  headers: { 'Content-Type': 'application/json' },
  withCredentials: true,
})

let accessToken: string | null = null

export function setAccessToken(token: string | null) {
  accessToken = token
}

export function getAccessToken(): string | null {
  return accessToken
}

client.interceptors.request.use((config) => {
  if (accessToken) {
    config.headers.Authorization = `Bearer ${accessToken}`
  }
  return config
})

let refreshPromise: Promise<string | null> | null = null

client.interceptors.response.use(
  (response) => response,
  async (error) => {
    const original = error.config
    if (error.response?.status === 401 && !original._retry) {
      original._retry = true

      if (!refreshPromise) {
        refreshPromise = client
          .post<{ access_token: string }>('/auth/refresh')
          .then((res) => {
            const token = res.data.access_token
            setAccessToken(token)
            return token
          })
          .catch(() => {
            setAccessToken(null)
            router.push({ name: 'login' })
            return null
          })
          .finally(() => {
            refreshPromise = null
          })
      }

      const token = await refreshPromise
      if (token) {
        original.headers.Authorization = `Bearer ${token}`
        return client(original)
      }
    }
    return Promise.reject(error)
  },
)

export default client
