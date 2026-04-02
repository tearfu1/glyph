<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import type { BookWithAuthor } from '@/types/book'
import type { ReviewWithUser } from '@/types/review'
import type { QuestionWithUser } from '@/types/question'
import type { ReadingStatusType } from '@/types/reading-status'
import { getBook } from '@/api/books'
import * as reviewsApi from '@/api/reviews'
import * as questionsApi from '@/api/questions'
import { getMyStatuses, setReadingStatus as apiSetReadingStatus } from '@/api/reading-status'
import Pagination from '@/components/Pagination.vue'

const route = useRoute()
const auth = useAuthStore()
const bookId = computed(() => route.params.id as string)

// Book
const book = ref<BookWithAuthor | null>(null)
const loading = ref(true)

// Reviews
const reviews = ref<ReviewWithUser[]>([])
const reviewsPage = ref(1)
const reviewsTotal = ref(0)
const reviewsPerPage = ref(10)
const myReview = ref<ReviewWithUser | null>(null)

// Review form
const reviewRating = ref(5)
const reviewText = ref('')
const reviewSubmitting = ref(false)
const editingReview = ref(false)

// Tabs
const activeTab = ref<'reviews' | 'questions'>('reviews')

// Reading status
const currentStatus = ref<ReadingStatusType | null>(null)
const statusLoading = ref(false)
const showStatusMenu = ref(false)

// Questions
const questions = ref<QuestionWithUser[]>([])
const questionsPage = ref(1)
const questionsTotal = ref(0)
const questionsPerPage = ref(10)
const questionText = ref('')
const questionSubmitting = ref(false)

const tagColors: Record<string, string> = {
  genre: 'bg-indigo-50 text-indigo-700',
  mood: 'bg-amber-50 text-amber-700',
  theme: 'bg-emerald-50 text-emerald-700',
  period: 'bg-slate-100 text-slate-600',
}

const statusLabels: Record<ReadingStatusType, string> = {
  want_to_read: 'Хочу прочитать',
  reading: 'Читаю',
  read: 'Прочитано',
}

const statusOptions: ReadingStatusType[] = ['want_to_read', 'reading', 'read']

const filteredReviews = computed(() => {
  if (!auth.user) return reviews.value
  return reviews.value.filter(r => r.user_id !== auth.user!.id)
})

const canAskQuestion = computed(() => {
  const role = auth.userRole
  return role === 'premium' || role === 'admin'
})

async function fetchBook() {
  loading.value = true
  try {
    const { data } = await getBook(bookId.value)
    book.value = data
  } catch {
    book.value = null
  } finally {
    loading.value = false
  }
}

async function fetchReviews() {
  try {
    const { data } = await reviewsApi.getReviews(bookId.value, reviewsPage.value)
    reviews.value = data.data
    reviewsTotal.value = data.total
    reviewsPerPage.value = data.per_page
  } catch {}
}

async function fetchMyReview() {
  if (!auth.isAuthenticated) return
  try {
    const { data } = await reviewsApi.getMyReview(bookId.value)
    myReview.value = data
  } catch {}
}

async function fetchQuestions() {
  try {
    const { data } = await questionsApi.getQuestions(bookId.value, questionsPage.value)
    questions.value = data.data
    questionsTotal.value = data.total
    questionsPerPage.value = data.per_page
  } catch {}
}

async function submitReview() {
  if (!reviewText.value.trim()) return
  reviewSubmitting.value = true
  try {
    if (editingReview.value && myReview.value) {
      await reviewsApi.updateReview(myReview.value.id, {
        rating: reviewRating.value,
        text: reviewText.value,
      })
    } else {
      await reviewsApi.createReview(bookId.value, {
        rating: reviewRating.value,
        text: reviewText.value,
      })
    }
    editingReview.value = false
    reviewText.value = ''
    reviewRating.value = 5
    await Promise.all([fetchMyReview(), fetchReviews()])
  } catch {
  } finally {
    reviewSubmitting.value = false
  }
}

function cancelReviewForm() {
  editingReview.value = false
  reviewText.value = ''
  reviewRating.value = 5
}

function startEdit() {
  if (!myReview.value) return
  reviewRating.value = myReview.value.rating
  reviewText.value = myReview.value.text
  editingReview.value = true
}

async function removeReview() {
  if (!myReview.value) return
  try {
    await reviewsApi.deleteReview(myReview.value.id)
    myReview.value = null
    editingReview.value = false
    await fetchReviews()
  } catch {}
}

async function toggleReaction(review: ReviewWithUser, isLike: boolean) {
  if (!auth.isAuthenticated) return
  try {
    if (review.user_reaction === isLike) {
      await reviewsApi.removeReaction(review.id)
    } else {
      await reviewsApi.addReaction(review.id, isLike)
    }
    await fetchReviews()
  } catch {}
}

async function fetchReadingStatus() {
  if (!auth.isAuthenticated) return
  try {
    const { data } = await getMyStatuses()
    const found = data.find(s => s.book_id === bookId.value)
    currentStatus.value = found?.status ?? null
  } catch {}
}

async function setStatus(status: ReadingStatusType) {
  statusLoading.value = true
  showStatusMenu.value = false
  try {
    await apiSetReadingStatus(bookId.value, status)
    currentStatus.value = status
  } catch {
  } finally {
    statusLoading.value = false
  }
}

async function toggleQuestionReaction(question: QuestionWithUser, isLike: boolean) {
  if (!auth.isAuthenticated) return
  try {
    if (question.user_reaction === isLike) {
      await questionsApi.removeReaction(question.id)
    } else {
      await questionsApi.addReaction(question.id, isLike)
    }
    await fetchQuestions()
  } catch {}
}

async function submitQuestion() {
  if (!questionText.value.trim()) return
  questionSubmitting.value = true
  try {
    await questionsApi.createQuestion(bookId.value, questionText.value)
    questionText.value = ''
    await fetchQuestions()
  } catch {
  } finally {
    questionSubmitting.value = false
  }
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('ru-RU', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
}

function closeStatusMenu(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest('.status-dropdown')) {
    showStatusMenu.value = false
  }
}

watch(reviewsPage, fetchReviews)
watch(questionsPage, fetchQuestions)

onMounted(() => {
  fetchBook()
  fetchReviews()
  fetchQuestions()
  if (auth.isAuthenticated) {
    fetchMyReview()
    fetchReadingStatus()
  }
  document.addEventListener('click', closeStatusMenu)
})

onUnmounted(() => {
  document.removeEventListener('click', closeStatusMenu)
})
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-20">
      <div class="w-8 h-8 border-4 border-indigo-200 border-t-indigo-600 rounded-full animate-spin" />
    </div>

    <!-- Not found -->
    <div v-else-if="!book" class="text-center py-20">
      <p class="text-lg text-gray-500">Книга не найдена</p>
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

      <!-- ===== Book Header ===== -->
      <div class="flex flex-col sm:flex-row gap-8">
        <!-- Cover -->
        <div class="w-48 sm:w-56 shrink-0 mx-auto sm:mx-0">
          <div class="aspect-[2/3] bg-gray-100 rounded-xl overflow-hidden shadow-md">
            <img
              v-if="book.cover_url"
              :src="book.cover_url"
              :alt="book.title"
              class="w-full h-full object-cover"
            />
            <div v-else class="w-full h-full flex items-center justify-center text-gray-400">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                  d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
              </svg>
            </div>
          </div>
        </div>

        <!-- Info -->
        <div class="flex-1 min-w-0">
          <h1 class="text-2xl sm:text-3xl font-bold text-gray-900">{{ book.title }}</h1>

          <!-- Author -->
          <div class="mt-3 flex items-center gap-3">
            <img
              v-if="book.author_avatar_url"
              :src="book.author_avatar_url"
              :alt="book.author_display_name"
              class="w-8 h-8 rounded-full object-cover"
            />
            <div
              v-else
              class="w-8 h-8 rounded-full bg-indigo-100 flex items-center justify-center text-indigo-600 text-sm font-semibold"
            >
              {{ book.author_display_name?.charAt(0)?.toUpperCase() }}
            </div>
            <span class="text-gray-700 font-medium">{{ book.author_display_name }}</span>
          </div>

          <!-- Meta -->
          <div class="mt-4 flex flex-wrap gap-x-6 gap-y-1 text-sm text-gray-500">
            <span v-if="book.published_year">
              <span class="text-gray-400">Год:</span> {{ book.published_year }}
            </span>
            <span v-if="book.isbn">
              <span class="text-gray-400">ISBN:</span> {{ book.isbn }}
            </span>
          </div>

          <!-- Tags -->
          <div v-if="book.tags?.length" class="mt-4 flex flex-wrap gap-2">
            <span
              v-for="tag in book.tags"
              :key="tag.id"
              class="px-2.5 py-1 text-xs font-medium rounded-lg"
              :class="tagColors[tag.tag_type] || 'bg-gray-100 text-gray-600'"
            >
              {{ tag.name }}
            </span>
          </div>

          <!-- Reading Status -->
          <div v-if="auth.isAuthenticated" class="mt-5 status-dropdown relative inline-block">
            <button
              @click.stop="showStatusMenu = !showStatusMenu"
              :disabled="statusLoading"
              class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium border transition-colors cursor-pointer disabled:opacity-50"
              :class="currentStatus
                ? 'bg-indigo-50 text-indigo-700 border-indigo-200 hover:bg-indigo-100'
                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'"
            >
              <!-- Bookmark icon -->
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
              </svg>
              {{ currentStatus ? statusLabels[currentStatus] : 'Отметить книгу' }}
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
              </svg>
            </button>
            <div
              v-if="showStatusMenu"
              class="absolute left-0 mt-1 w-52 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-10"
            >
              <button
                v-for="opt in statusOptions"
                :key="opt"
                @click="setStatus(opt)"
                class="w-full text-left px-4 py-2.5 text-sm transition-colors cursor-pointer hover:bg-gray-50"
                :class="currentStatus === opt ? 'text-indigo-600 font-medium bg-indigo-50/50' : 'text-gray-700'"
              >
                {{ statusLabels[opt] }}
              </button>
            </div>
          </div>

          <!-- Description -->
          <p v-if="book.description" class="mt-6 text-gray-700 leading-relaxed whitespace-pre-line">
            {{ book.description }}
          </p>
        </div>
      </div>

      <!-- ===== Tabs ===== -->
      <section class="mt-14 pb-12">
        <!-- Tab bar -->
        <div class="border-b border-gray-200 mb-6">
          <nav class="flex gap-8">
            <button
              @click="activeTab = 'reviews'"
              class="pb-3 text-sm font-medium border-b-2 transition-colors cursor-pointer"
              :class="activeTab === 'reviews'
                ? 'border-indigo-600 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'"
            >
              Рецензии
              <span v-if="reviewsTotal" class="ml-1.5 px-2 py-0.5 text-xs rounded-full" :class="activeTab === 'reviews' ? 'bg-indigo-100 text-indigo-600' : 'bg-gray-100 text-gray-500'">{{ reviewsTotal }}</span>
            </button>
            <button
              @click="activeTab = 'questions'"
              class="pb-3 text-sm font-medium border-b-2 transition-colors cursor-pointer"
              :class="activeTab === 'questions'
                ? 'border-indigo-600 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'"
            >
              Вопросы автору
              <span v-if="questionsTotal" class="ml-1.5 px-2 py-0.5 text-xs rounded-full" :class="activeTab === 'questions' ? 'bg-indigo-100 text-indigo-600' : 'bg-gray-100 text-gray-500'">{{ questionsTotal }}</span>
            </button>
          </nav>
        </div>

        <!-- ===== Reviews Tab ===== -->
        <div v-if="activeTab === 'reviews'">
          <!-- Auth prompt -->
          <p v-if="!auth.isAuthenticated" class="text-sm text-gray-500 mb-6">
            <RouterLink :to="{ name: 'login' }" class="text-indigo-600 hover:text-indigo-500">Войдите</RouterLink>, чтобы оставить рецензию
          </p>

          <!-- My review card -->
          <div
            v-if="myReview && !editingReview"
            class="mb-6 p-5 bg-indigo-50/50 border border-indigo-100 rounded-xl"
          >
            <div class="flex items-start justify-between gap-4">
              <div>
                <span class="text-xs font-semibold text-indigo-600 uppercase tracking-wide">Ваша рецензия</span>
                <div class="flex gap-0.5 mt-1">
                  <svg
                    v-for="i in 5"
                    :key="i"
                    xmlns="http://www.w3.org/2000/svg"
                    class="w-5 h-5"
                    :class="i <= myReview.rating ? 'text-amber-400' : 'text-gray-300'"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.006 5.404.434c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.434 2.082-5.005Z" clip-rule="evenodd" />
                  </svg>
                </div>
              </div>
              <div class="flex gap-3 shrink-0">
                <button @click="startEdit" class="text-sm text-gray-500 hover:text-indigo-600 transition-colors cursor-pointer">
                  Редактировать
                </button>
                <button @click="removeReview" class="text-sm text-gray-500 hover:text-red-600 transition-colors cursor-pointer">
                  Удалить
                </button>
              </div>
            </div>
            <p class="mt-3 text-gray-700 text-sm leading-relaxed whitespace-pre-line">{{ myReview.text }}</p>
            <p class="mt-2 text-xs text-gray-400">{{ formatDate(myReview.created_at) }}</p>
          </div>

          <!-- Review form -->
          <div
            v-if="auth.isAuthenticated && (!myReview || editingReview)"
            class="mb-6 p-5 bg-white border border-gray-200 rounded-xl"
          >
            <h3 class="text-sm font-semibold text-gray-900 mb-4">
              {{ editingReview ? 'Редактирование рецензии' : 'Новая рецензия' }}
            </h3>

            <!-- Star picker -->
            <div class="flex items-center gap-3 mb-4">
              <span class="text-sm text-gray-600">Оценка:</span>
              <div class="flex gap-1">
                <button
                  v-for="i in 5"
                  :key="i"
                  @click="reviewRating = i"
                  class="cursor-pointer p-0.5 rounded hover:scale-110 transition-transform"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="w-7 h-7 transition-colors"
                    :class="i <= reviewRating ? 'text-amber-400' : 'text-gray-300 hover:text-amber-200'"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.006 5.404.434c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.434 2.082-5.005Z" clip-rule="evenodd" />
                  </svg>
                </button>
              </div>
            </div>

            <textarea
              v-model="reviewText"
              rows="4"
              placeholder="Поделитесь впечатлениями о книге..."
              class="w-full px-4 py-3 border border-gray-300 rounded-lg text-sm resize-y focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
            />

            <div class="mt-3 flex gap-3">
              <button
                @click="submitReview"
                :disabled="reviewSubmitting || !reviewText.trim()"
                class="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ reviewSubmitting ? 'Сохранение...' : (editingReview ? 'Сохранить' : 'Опубликовать') }}
              </button>
              <button
                v-if="editingReview"
                @click="cancelReviewForm"
                class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800 transition-colors cursor-pointer"
              >
                Отмена
              </button>
            </div>
          </div>

          <!-- Reviews list -->
          <div v-if="filteredReviews.length" class="space-y-4">
            <div
              v-for="review in filteredReviews"
              :key="review.id"
              class="p-5 bg-white border border-gray-200 rounded-xl"
            >
              <div class="flex items-start justify-between gap-4">
                <div class="flex items-center gap-3">
                  <img
                    v-if="review.user_avatar_url"
                    :src="review.user_avatar_url"
                    :alt="review.user_display_name"
                    class="w-8 h-8 rounded-full object-cover"
                  />
                  <div
                    v-else
                    class="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-500 text-sm font-semibold"
                  >
                    {{ review.user_display_name?.charAt(0)?.toUpperCase() }}
                  </div>
                  <div>
                    <span class="text-sm font-medium text-gray-900">{{ review.user_display_name }}</span>
                    <div class="flex gap-0.5 mt-0.5">
                      <svg
                        v-for="i in 5"
                        :key="i"
                        xmlns="http://www.w3.org/2000/svg"
                        class="w-3.5 h-3.5"
                        :class="i <= review.rating ? 'text-amber-400' : 'text-gray-300'"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.006 5.404.434c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.434 2.082-5.005Z" clip-rule="evenodd" />
                      </svg>
                    </div>
                  </div>
                </div>
                <span class="text-xs text-gray-400 shrink-0">{{ formatDate(review.created_at) }}</span>
              </div>

              <p class="mt-3 text-sm text-gray-700 leading-relaxed whitespace-pre-line">{{ review.text }}</p>

              <!-- Reactions -->
              <div class="mt-3 flex items-center gap-4">
                <button
                  @click="toggleReaction(review, true)"
                  class="inline-flex items-center gap-1.5 text-sm transition-colors cursor-pointer"
                  :class="review.user_reaction === true ? 'text-indigo-600' : 'text-gray-400 hover:text-indigo-600'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M14 10h4.764a2 2 0 011.789 2.894l-3.5 7A2 2 0 0115.263 21h-4.017c-.163 0-.326-.02-.485-.06L7 20m7-10V5a2 2 0 00-2-2h-.095c-.5 0-.905.405-.905.905 0 .714-.211 1.412-.608 2.006L7 11v9m7-10h-2M7 20H5a2 2 0 01-2-2v-6a2 2 0 012-2h2.5" />
                  </svg>
                  <span v-if="review.like_count">{{ review.like_count }}</span>
                </button>
                <button
                  @click="toggleReaction(review, false)"
                  class="inline-flex items-center gap-1.5 text-sm transition-colors cursor-pointer"
                  :class="review.user_reaction === false ? 'text-red-500' : 'text-gray-400 hover:text-red-500'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M10 14H5.236a2 2 0 01-1.789-2.894l3.5-7A2 2 0 018.736 3h4.018c.163 0 .326.02.485.06L17 4m-7 10v2a3.5 3.5 0 003.5 3.5h.095c.5 0 .905-.405.905-.905 0-.714.211-1.412.608-2.006L17 13V4m-7 10h2m5-6h2a2 2 0 012 2v6a2 2 0 01-2 2h-2.5" />
                  </svg>
                  <span v-if="review.dislike_count">{{ review.dislike_count }}</span>
                </button>
              </div>
            </div>
          </div>

          <div v-else-if="!loading" class="text-center py-10 text-gray-400 text-sm">
            Рецензий пока нет
          </div>

          <div class="mt-6">
            <Pagination v-model:page="reviewsPage" :total="reviewsTotal" :per-page="reviewsPerPage" />
          </div>
        </div>

        <!-- ===== Questions Tab ===== -->
        <div v-if="activeTab === 'questions'">
          <!-- Question form (premium / admin only) -->
          <div v-if="canAskQuestion" class="mb-6">
            <textarea
              v-model="questionText"
              rows="3"
              placeholder="Задайте вопрос автору..."
              class="w-full px-4 py-3 border border-gray-300 rounded-lg text-sm resize-y focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
            />
            <button
              @click="submitQuestion"
              :disabled="questionSubmitting || !questionText.trim()"
              class="mt-2 px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {{ questionSubmitting ? 'Отправка...' : 'Задать вопрос' }}
            </button>
          </div>

          <!-- Questions list -->
          <div v-if="questions.length" class="space-y-4">
            <div
              v-for="question in questions"
              :key="question.id"
              class="p-5 bg-white border border-gray-200 rounded-xl"
            >
              <div class="flex items-start justify-between gap-4">
                <div class="flex items-center gap-3">
                  <img
                    v-if="question.user_avatar_url"
                    :src="question.user_avatar_url"
                    :alt="question.user_display_name"
                    class="w-8 h-8 rounded-full object-cover"
                  />
                  <div
                    v-else
                    class="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-500 text-sm font-semibold"
                  >
                    {{ question.user_display_name?.charAt(0)?.toUpperCase() }}
                  </div>
                  <span class="text-sm font-medium text-gray-900">{{ question.user_display_name }}</span>
                  <span
                    v-if="question.has_answer"
                    class="px-2 py-0.5 text-[10px] font-semibold rounded-full bg-emerald-50 text-emerald-700"
                  >
                    Есть ответ
                  </span>
                </div>
                <span class="text-xs text-gray-400 shrink-0">{{ formatDate(question.created_at) }}</span>
              </div>
              <p class="mt-3 text-sm text-gray-700 leading-relaxed whitespace-pre-line">{{ question.text }}</p>

              <!-- Answer -->
              <div v-if="question.answer_text" class="mt-4 ml-4 pl-4 border-l-2 border-indigo-200">
                <div class="flex items-center gap-2 mb-1.5">
                  <img
                    v-if="question.answer_user_avatar_url"
                    :src="question.answer_user_avatar_url"
                    :alt="question.answer_user_display_name ?? ''"
                    class="w-6 h-6 rounded-full object-cover"
                  />
                  <div
                    v-else-if="question.answer_user_display_name"
                    class="w-6 h-6 rounded-full bg-indigo-100 flex items-center justify-center text-indigo-600 text-[10px] font-semibold"
                  >
                    {{ question.answer_user_display_name.charAt(0).toUpperCase() }}
                  </div>
                  <span class="text-xs font-medium text-indigo-600">{{ question.answer_user_display_name }}</span>
                  <span class="text-[10px] text-gray-400 uppercase tracking-wide font-semibold">Автор</span>
                  <span v-if="question.answer_created_at" class="text-xs text-gray-400 ml-auto">{{ formatDate(question.answer_created_at) }}</span>
                </div>
                <p class="text-sm text-gray-700 leading-relaxed whitespace-pre-line">{{ question.answer_text }}</p>
              </div>

              <div class="mt-3 flex items-center gap-4">
                <button
                  @click="toggleQuestionReaction(question, true)"
                  class="inline-flex items-center gap-1.5 text-sm transition-colors cursor-pointer"
                  :class="question.user_reaction === true ? 'text-indigo-600' : 'text-gray-400 hover:text-indigo-600'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M14 10h4.764a2 2 0 011.789 2.894l-3.5 7A2 2 0 0115.263 21h-4.017c-.163 0-.326-.02-.485-.06L7 20m7-10V5a2 2 0 00-2-2h-.095c-.5 0-.905.405-.905.905 0 .714-.211 1.412-.608 2.006L7 11v9m7-10h-2M7 20H5a2 2 0 01-2-2v-6a2 2 0 012-2h2.5" />
                  </svg>
                  <span v-if="question.like_count">{{ question.like_count }}</span>
                </button>
                <button
                  @click="toggleQuestionReaction(question, false)"
                  class="inline-flex items-center gap-1.5 text-sm transition-colors cursor-pointer"
                  :class="question.user_reaction === false ? 'text-red-500' : 'text-gray-400 hover:text-red-500'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M10 14H5.236a2 2 0 01-1.789-2.894l3.5-7A2 2 0 018.736 3h4.018c.163 0 .326.02.485.06L17 4m-7 10v2a3.5 3.5 0 003.5 3.5h.095c.5 0 .905-.405.905-.905 0-.714.211-1.412.608-2.006L17 13V4m-7 10h2m5-6h2a2 2 0 012 2v6a2 2 0 01-2 2h-2.5" />
                  </svg>
                  <span v-if="question.dislike_count">{{ question.dislike_count }}</span>
                </button>
              </div>
            </div>
          </div>

          <div v-else-if="!loading" class="text-center py-10 text-gray-400 text-sm">
            Вопросов пока нет
          </div>

          <div class="mt-6">
            <Pagination v-model:page="questionsPage" :total="questionsTotal" :per-page="questionsPerPage" />
          </div>
        </div>
      </section>
    </template>
  </div>
</template>
