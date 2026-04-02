<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import type { PublicUser } from '@/types/user'
import type { ReviewWithUser } from '@/types/review'
import type { BookWithAuthor } from '@/types/book'
import type { ReadingStatusType } from '@/types/reading-status'
import { getUserProfile, getUserReviews } from '@/api/users'
import { getBooks } from '@/api/books'
import * as reviewsApi from '@/api/reviews'
import { useReactions } from '@/composables/useReactions'
import ReviewCard from '@/components/ReviewCard.vue'
import Pagination from '@/components/Pagination.vue'

const route = useRoute()
const auth = useAuthStore()
const userId = computed(() => route.params.id as string)

const user = ref<PublicUser | null>(null)
const loading = ref(true)

// Reviews
const reviews = ref<ReviewWithUser[]>([])
const reviewsPage = ref(1)
const reviewsTotal = ref(0)
const reviewsPerPage = ref(20)

// Shelf
const shelfTab = ref<ReadingStatusType>('reading')
const shelfBooks = ref<BookWithAuthor[]>([])
const shelfTotal = ref(0)
const shelfPage = ref(1)
const shelfPerPage = ref(20)
const shelfLoading = ref(false)

const isOwnProfile = computed(() => auth.user?.id === userId.value)

const roleLabels: Record<string, string> = {
  user: 'Читатель',
  premium: 'Премиум',
  author: 'Автор',
  admin: 'Администратор',
}

const roleBadge: Record<string, string> = {
  user: 'bg-gray-100 text-gray-600',
  premium: 'bg-amber-50 text-amber-700',
  author: 'bg-indigo-50 text-indigo-700',
  admin: 'bg-red-50 text-red-700',
}

const shelfTabs: { key: ReadingStatusType; label: string }[] = [
  { key: 'reading', label: 'Читаю' },
  { key: 'want_to_read', label: 'Хочу прочитать' },
  { key: 'read', label: 'Прочитано' },
]

const { toggleReaction: onReviewReact } = useReactions(
  reviewsApi.addReaction,
  reviewsApi.removeReaction,
  fetchReviews,
)

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
}

async function fetchUser() {
  loading.value = true
  try {
    const { data } = await getUserProfile(userId.value)
    user.value = data
  } catch {
    user.value = null
  } finally {
    loading.value = false
  }
}

async function fetchReviews() {
  try {
    const { data } = await getUserReviews(userId.value, reviewsPage.value)
    reviews.value = data.data
    reviewsTotal.value = data.total
    reviewsPerPage.value = data.per_page
  } catch {}
}

async function fetchShelf() {
  shelfLoading.value = true
  try {
    const { data } = await getBooks({
      page: shelfPage.value,
      // Backend endpoint: GET /books/shelf/:userId?status=&page=
    })
    // TODO: connect to /books/shelf/:userId when shelf API is wired
    shelfBooks.value = data.data
    shelfTotal.value = data.total
    shelfPerPage.value = data.per_page
  } catch {
  } finally {
    shelfLoading.value = false
  }
}

function handleReaction(review: ReviewWithUser, isLike: boolean) {
  onReviewReact(review, isLike)
}

watch(reviewsPage, fetchReviews)
watch(userId, () => {
  reviewsPage.value = 1
  fetchUser()
  fetchReviews()
})

onMounted(() => {
  fetchUser()
  fetchReviews()
})
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-20">
      <div class="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin" />
    </div>

    <!-- Not found -->
    <div v-else-if="!user" class="text-center py-20">
      <p class="text-lg text-gray-500">Пользователь не найден</p>
      <RouterLink to="/" class="mt-4 inline-block text-sm text-indigo-600 hover:text-indigo-500">
        &larr; Вернуться в каталог
      </RouterLink>
    </div>

    <template v-else>
      <!-- Back -->
      <RouterLink to="/" class="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-indigo-600 transition-colors mb-6">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
        </svg>
        Каталог
      </RouterLink>

      <!-- ===== User Header ===== -->
      <div class="flex items-center gap-6 mb-10">
        <img
          v-if="user.avatar_url"
          :src="user.avatar_url"
          :alt="user.display_name"
          class="w-20 h-20 rounded-full object-cover shadow-md"
        />
        <div
          v-else
          class="w-20 h-20 rounded-full bg-indigo-100 flex items-center justify-center text-indigo-600 text-2xl font-bold shadow-md"
        >
          {{ user.display_name?.charAt(0)?.toUpperCase() }}
        </div>
        <div>
          <h1 class="text-2xl font-bold text-gray-900">{{ user.display_name }}</h1>
          <p class="text-sm text-gray-500 mt-1">@{{ user.login }}</p>
          <div class="mt-2 flex items-center gap-3">
            <span
              class="px-2.5 py-1 text-xs font-medium rounded-lg"
              :class="roleBadge[user.role] || 'bg-gray-100 text-gray-600'"
            >
              {{ roleLabels[user.role] || user.role }}
            </span>
            <span class="text-xs text-gray-400">
              На платформе с {{ formatDate(user.created_at) }}
            </span>
          </div>
          <RouterLink
            v-if="isOwnProfile"
            :to="{ name: 'settings' }"
            class="mt-3 inline-flex items-center gap-1.5 text-sm text-indigo-600 hover:text-indigo-500 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Настройки
          </RouterLink>
        </div>
      </div>

      <!-- ===== Reviews Section ===== -->
      <section>
        <h2 class="text-lg font-semibold text-gray-900 mb-4">
          Рецензии
          <span v-if="reviewsTotal" class="ml-2 px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-500">{{ reviewsTotal }}</span>
        </h2>

        <div v-if="reviews.length" class="space-y-4">
          <div v-for="review in reviews" :key="review.id">
            <RouterLink
              :to="{ name: 'book', params: { id: review.book_id } }"
              class="text-xs text-indigo-600 hover:text-indigo-500 mb-1 inline-block"
            >
              Перейти к книге &rarr;
            </RouterLink>
            <ReviewCard
              :review="review"
              @react="handleReaction(review, $event)"
            />
          </div>
        </div>

        <div v-else class="text-center py-10 text-gray-400 text-sm">
          Рецензий пока нет
        </div>

        <div class="mt-6">
          <Pagination v-model:page="reviewsPage" :total="reviewsTotal" :per-page="reviewsPerPage" />
        </div>
      </section>
    </template>
  </div>
</template>
