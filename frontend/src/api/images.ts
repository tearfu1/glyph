import client from './client'

export interface UploadResponse {
  url: string
}

export function uploadImage(file: File) {
  const form = new FormData()
  form.append('file', file)
  return client.post<UploadResponse>('/images/upload', form, {
    headers: { 'Content-Type': 'multipart/form-data' },
  })
}
