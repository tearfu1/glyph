import client from './client'
import type { AuthResponse, LoginRequest, RegisterRequest } from '@/types/user'

export function login(data: LoginRequest) {
  return client.post<AuthResponse>('/auth/login', data)
}

export function register(data: RegisterRequest) {
  return client.post<AuthResponse>('/auth/register', data)
}

export function refresh() {
  return client.post<AuthResponse>('/auth/refresh')
}

export function logout() {
  return client.post('/auth/logout')
}
