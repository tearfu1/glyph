export type UserRole = 'user' | 'premium' | 'author' | 'admin'

export interface PublicUser {
  id: string
  login: string
  email: string
  display_name: string
  role: UserRole
  avatar_url: string | null
  created_at: string
}

export interface AuthResponse {
  access_token: string
  user: PublicUser
}

export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  login: string
  email: string
  password: string
  display_name: string
}
