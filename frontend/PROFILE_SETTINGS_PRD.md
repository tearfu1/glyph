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

- Обобщённая логика toggle-реакций, заменяет дублирование `toggleReaction` / `toggleQuestionReaction` в BookDetails
- Принимает `addReaction`, `removeReaction`, `onUpdate` callback
- Возвращает `toggleReaction(item, isLike)` — проверяет авторизацию, toggle/remove по `user_reaction`, вызывает refresh
- Используется в BookDetails.vue (2 экземпляра: reviews + questions) и UserProfile.vue (reviews)

### 3. Компонент ReactionButtons.vue

**Файл:** `frontend/src/components/ReactionButtons.vue`

- Props: `likeCount`, `dislikeCount`, `userReaction` (boolean | null)
- Emit: `react(isLike: boolean)`
- Кнопки с SVG-иконками (thumbs-up/down), подсветка активной реакции (indigo/red)
- Счётчики скрываются при нулевом значении

### 4. Компонент StarRating.vue

**Файл:** `frontend/src/components/StarRating.vue`

- Props: `modelValue` (1-5), `readonly` (boolean), `size` ('sm' | 'md' | 'lg')
- Emit: `update:modelValue` (только если не readonly)
- Режим readonly: `<span>`, без hover-эффектов — для отображения рейтинга
- Режим интерактивный: `<button>`, hover-scale, hover-color — для формы
- Три размера: sm (3.5), md (5), lg (7)

### 5. Компонент ReviewCard.vue

**Файл:** `frontend/src/components/ReviewCard.vue`

- Props: `review: ReviewWithUser`
- Emit: `react(isLike: boolean)`
- Аватар (изображение или инициал) → ссылка на профиль `/user/:id`
- Имя пользователя → ссылка на профиль
- StarRating (readonly, sm)
- Текст рецензии, дата
- ReactionButtons
- Используется в BookDetails.vue и UserProfile.vue

### 6. Страница UserProfile.vue

**Файл:** `frontend/src/views/UserProfile.vue`  
**Маршрут:** `/user/:id` (name: `user`)

- Загрузка профиля через `getUserProfile(userId)` → PublicUser
- Шапка: аватар 80px, display_name, @login, бейдж роли (цветовая кодировка), дата регистрации
- Кнопка «Настройки» видна только на своём профиле (`isOwnProfile`)
- Секция рецензий: пагинация, ReviewCard, ссылка «Перейти к книге» над каждой карточкой
- Watch на `userId` — перезагрузка при смене маршрута
- Спиннер, заглушка «Пользователь не найден», ссылка «← Каталог»

### 7. Страница SettingsView.vue

**Файл:** `frontend/src/views/SettingsView.vue`  
**Маршрут:** `/settings` (name: `settings`, meta: `requiresAuth`)

- Загрузка через `getMySettings()` → заполняет форму
- Превью аватара (изображение или инициал), логин, email (readonly)
- Поля: display_name (required, maxlength 128), avatar_url (type=url, подсказка)
- Сохранение через `updateMe()` → обновляет `auth.user` в Pinia (хедер обновляется реактивно)
- Состояния: loading, saving, success, error
- Валидация: кнопка disabled если пустое имя или идёт сохранение

### 8. Маршруты и навигация

**Файл:** `frontend/src/router/index.ts`

- `/user/:id` → UserProfile (lazy import)
- `/settings` → SettingsView (lazy import, `meta: { requiresAuth: true }`)
- Guard `requiresAuth` уже был реализован в `beforeEach`

**Файл:** `frontend/src/components/layout/AppHeader.vue`

- Имя пользователя → ссылка на свой профиль `/user/:id`
- Иконка шестерёнки → ссылка на `/settings`
- Кнопка «Выйти» — без изменений

### 9. Рефакторинг BookDetails.vue

**Файл:** `frontend/src/views/BookDetails.vue`

Заменено:
- Inline звёзды рейтинга → `<StarRating>` (пикер в форме + readonly в карточке «Ваша рецензия»)
- Inline кнопки лайк/дизлайк для рецензий и вопросов → `<ReactionButtons>`
- Inline карточки рецензий → `<ReviewCard>`
- Дублированные `toggleReaction` / `toggleQuestionReaction` → `useReactions` composable (2 экземпляра)
- Имя автора книги → ссылка на профиль `/user/:authorId`
- Аватары пользователей в вопросах → ссылки на профили

Не затронуто (сохранено как было):
- Форма рецензии, «Ваша рецензия», чтение-статус dropdown, табы, форма вопроса, карточки вопросов с ответами

---

## Созданные файлы

```
frontend/src/api/users.ts
frontend/src/composables/useReactions.ts
frontend/src/components/ReactionButtons.vue
frontend/src/components/StarRating.vue
frontend/src/components/ReviewCard.vue
frontend/src/views/UserProfile.vue
frontend/src/views/SettingsView.vue
```

## Изменённые файлы

```
frontend/src/router/index.ts           — +2 маршрута
frontend/src/components/layout/AppHeader.vue — навигация профиль + настройки
frontend/src/views/BookDetails.vue      — рефакторинг на компоненты + composable
```

## Проверки

- `vue-tsc --noEmit` — 0 ошибок
- `vite build` — успешно, 116 модулей
