# Профиль, настройки, компоненты — PRD

## Статус: реализовано

---

## Что реализовано

### 1. API-модуль users.ts

**Файл:** `frontend/src/api/users.ts`

- `getUserProfile(userId)` → `GET /api/users/:id/profile` → `PublicUser`
- `getUserReviews(userId, page)` → `GET /api/users/:id/reviews?page=` → `PaginatedResponse<ReviewWithUser>`
- `getMySettings()` → `GET /api/users/me/settings` → `PublicUser`
- `updateMe({ display_name?, avatar_url? })` → `PUT /api/users/me` → `PublicUser`

### 2. Composable useReactions.ts

**Файл:** `frontend/src/composables/useReactions.ts`

- Обобщённая логика toggle-реакций, заменяет дублирование
- Принимает `addReaction`, `removeReaction`, `onUpdate` callback
- Возвращает `toggleReaction(item, isLike)`
- Используется в BookDetails.vue (reviews + questions) и UserProfile.vue (reviews)

### 3. Composable usePagination.ts

**Файл:** `frontend/src/composables/usePagination.ts`

- Обобщённый паттерн пагинации: `usePagination(fetchFn)` → `{ items, page, total, perPage, loading, load, changePage }`
- Автоматический watch на `page` → вызов `load()`
- fetchFn принимает page, возвращает `Promise<{ data: PaginatedResponse<T> }>`
- Используется в HomeView (books), BookDetails (reviews, questions), UserProfile (reviews, shelf)
- Заменяет дублирование из ~6 мест

### 4. Компонент ReactionButtons.vue

**Файл:** `frontend/src/components/ReactionButtons.vue`

- Props: `likeCount`, `dislikeCount`, `userReaction` (boolean | null)
- Emit: `react(isLike: boolean)`
- Кнопки с SVG-иконками, подсветка активной реакции

### 5. Компонент StarRating.vue

**Файл:** `frontend/src/components/StarRating.vue`

- Props: `modelValue` (1-5), `readonly`, `size` ('sm' | 'md' | 'lg')
- Режим readonly и интерактивный

### 6. Компонент ReviewCard.vue

**Файл:** `frontend/src/components/ReviewCard.vue`

- Props: `review: ReviewWithUser`
- Emit: `react(isLike: boolean)`
- Аватар, имя (ссылка на профиль), StarRating, текст, дата, ReactionButtons

### 7. Компонент QuestionCard.vue

**Файл:** `frontend/src/components/QuestionCard.vue`

- Props: `question: QuestionWithUser`
- Emit: `react(isLike: boolean)`
- Аватар, имя (ссылка на профиль), текст, дата, бейдж «Есть ответ»
- Блок ответа автора: indigo-полоска слева, аватар + имя, бейдж «Автор», дата, текст
- ReactionButtons
- Вынесен из BookDetails.vue, используется там же

### 8. Страница UserProfile.vue

**Файл:** `frontend/src/views/UserProfile.vue`
**Маршрут:** `/user/:id` (name: `user`)

- Загрузка профиля через `getUserProfile(userId)` → PublicUser
- Шапка: аватар 80px, display_name, @login, бейдж роли, дата регистрации
- Кнопка «Настройки» видна только на своём профиле
- **Табы:** «Рецензии» / «Полка книг» с бейджами-счётчиками
- **Рецензии:** пагинация через `usePagination`, ReviewCard, ссылка «Перейти к книге»
- **Полка книг:** под-табы «Читаю / Хочу прочитать / Прочитано», BookCard в grid-сетке, пагинация
  - API: `getShelfBooks(userId, status, page)` → `GET /api/books/shelf/:userId?status=&page=`
  - Бэкенд возвращает `BookWithTags` (книга + автор + теги) — для полноценного отображения BookCard
- Watch на `userId` — перезагрузка при смене маршрута

### 9. Страница SettingsView.vue

**Файл:** `frontend/src/views/SettingsView.vue`
**Маршрут:** `/settings` (name: `settings`, meta: `requiresAuth`)

- Загрузка через `getMySettings()`, поля: display_name, avatar_url
- Сохранение через `updateMe()` → обновляет auth store
- Состояния: loading, saving, success, error

### 10. Маршруты и навигация

- `/user/:id` → UserProfile (lazy import)
- `/settings` → SettingsView (lazy import, `meta: { requiresAuth: true }`)
- AppHeader: ссылки на профиль и настройки

---

## Созданные файлы

```
frontend/src/api/users.ts
frontend/src/composables/useReactions.ts
frontend/src/composables/usePagination.ts
frontend/src/components/ReactionButtons.vue
frontend/src/components/StarRating.vue
frontend/src/components/ReviewCard.vue
frontend/src/components/QuestionCard.vue
frontend/src/views/UserProfile.vue
frontend/src/views/SettingsView.vue
```

## Изменённые файлы

```
frontend/src/router/index.ts               — +2 маршрута
frontend/src/components/layout/AppHeader.vue — навигация профиль + настройки
frontend/src/views/BookDetails.vue          — рефакторинг: usePagination + QuestionCard + useReactions
frontend/src/views/HomeView.vue             — рефакторинг: usePagination
frontend/src/api/books.ts                   — + getShelfBooks
frontend/src/api/reading-status.ts          — + deleteReadingStatus
```

## Проверки

- `vue-tsc --noEmit` — 0 ошибок
- `docker compose up -d --build backend` — успешно
