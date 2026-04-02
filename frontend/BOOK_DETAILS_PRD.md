# Страница книги (BookDetails) — PRD

## Статус: реализовано

---

## Что реализовано

### 1. Основная информация о книге

**Файлы:** `frontend/src/views/BookDetails.vue`, `frontend/src/api/books.ts`

- Загрузка данных через `GET /api/books/:id` → `BookWithTags` (книга + теги)
- Обложка (aspect 2:3) с плейсхолдером-иконкой
- Заголовок, автор (аватар-кружок + display_name), год, ISBN
- Описание с `whitespace-pre-line`
- Теги с цветовой кодировкой: genre=indigo, mood=amber, theme=emerald, period=slate
- Ссылка «<- Каталог» наверху
- Спиннер при загрузке, заглушка «Книга не найдена»

### 2. Статус чтения

**Файлы:**
- Бэкенд: `handlers/reading_status.rs`, `services/reading_status.rs`, `main.rs`
- Фронтенд: `api/reading-status.ts`, `types/reading-status.ts`, `views/BookDetails.vue`, `views/HomeView.vue`, `components/BookCard.vue`

**Эндпоинты:**
- `POST /api/books/:id/reading-status` — установить статус `{ status: "want_to_read" | "reading" | "read" }`
- `DELETE /api/books/:id/reading-status` — удалить статус чтения
- `GET /api/reading-statuses/my` — возвращает `Vec<ReadingStatus>` текущего пользователя

**Поведение:**
- На странице книги: dropdown-кнопка (Хочу прочитать / Читаю / Прочитано), виден только авторизованным
- В каталоге: статус отображается на каждой карточке BookCard; загружается одним запросом `getMyStatuses()` для всех книг
- Смена статуса из каталога через dropdown без перехода на страницу
- Оптимистичное обновление UI
- Цвета: want_to_read=amber, reading=indigo, read=emerald

### 3. Рецензии

**Файлы:** `api/reviews.ts`, `types/review.ts`, `views/BookDetails.vue`, `components/ReviewCard.vue`

**Эндпоинты:**
- `GET /api/books/:id/reviews?page=&per_page=` → `PaginatedResponse<ReviewWithUser>` (per_page: 1..100, default 20)
- `GET /api/books/:id/reviews/my` → `ReviewWithUser | null`
- `POST /api/books/:id/reviews` → `Review` (rating 1-5, text)
- `PUT /api/reviews/:id` → `Review`
- `DELETE /api/reviews/:id`
- `POST /api/reviews/:id/reaction` → `ReviewReaction` (is_like: bool)
- `DELETE /api/reviews/:id/reaction`

**Поведение:**
- Табы «Рецензии» / «Вопросы автору» с бейджами-счётчиками
- Пагинация через `usePagination` composable
- `OptionalAuthUser` на бэкенде — `user_reaction` возвращается для авторизованных
- Форма написания рецензии, редактирование, удаление
- `ReviewCard` — отдельный компонент (аватар, имя, звёзды, текст, дата, like/dislike)

### 4. Вопросы автору

**Файлы:** `api/questions.ts`, `types/question.ts`, `views/BookDetails.vue`, `components/QuestionCard.vue`

**Эндпоинты:**
- `GET /api/books/:id/questions?page=&per_page=` → `PaginatedResponse<QuestionWithUser>` (per_page: 1..100, default 20)
- `POST /api/books/:id/questions` → `Question` (только premium/admin)
- `POST /api/questions/:id/answer` → `Answer` (только author/admin книги)

**Поведение:**
- Форма вопроса видна для premium/admin
- `QuestionCard` — отдельный компонент (аватар, имя, текст, дата, бейдж «Есть ответ», реакции, блок ответа автора)
- Пагинация через `usePagination` composable
- `OptionalAuthUser` — `user_reaction` возвращается для авторизованных

### 5. Composables

- `useReactions(addReaction, removeReaction, onUpdate)` — toggle-логика лайков/дизлайков
- `usePagination(fetchFn)` — обобщённая пагинация: `{ items, page, total, perPage, loading, load, changePage }`. Используется в BookDetails (reviews, questions), HomeView (books), UserProfile (reviews, shelf)

---

## Созданные / изменённые файлы

```
frontend/src/types/review.ts
frontend/src/types/question.ts
frontend/src/types/reading-status.ts
frontend/src/api/reviews.ts
frontend/src/api/questions.ts
frontend/src/api/reading-status.ts          — + deleteReadingStatus
frontend/src/api/books.ts                   — + getShelfBooks
frontend/src/composables/useReactions.ts
frontend/src/composables/usePagination.ts   — NEW
frontend/src/components/QuestionCard.vue    — NEW (вынесен из BookDetails)
frontend/src/components/ReviewCard.vue
frontend/src/components/ReactionButtons.vue
frontend/src/components/StarRating.vue
frontend/src/views/BookDetails.vue          — рефакторинг: usePagination + QuestionCard
frontend/src/views/HomeView.vue             — рефакторинг: usePagination
```

**Бэкенд:**
```
backend/src/handlers/reviews.rs             — + per_page query param
backend/src/handlers/questions.rs           — + per_page query param
backend/src/handlers/books.rs               — get_book возвращает BookWithTags, get_shelf возвращает BookWithTags + BookWithAuthor
backend/src/handlers/reading_status.rs      — + delete_status handler
backend/src/services/review.rs              — per_page param
backend/src/services/question.rs            — per_page param
backend/src/services/book.rs                — get_books_for_shelf возвращает BookWithAuthor (JOIN author)
backend/src/services/reading_status.rs      — + delete_reading_status
backend/src/main.rs                         — + DELETE /api/books/:bookId/reading-status
```

---

## Ранее нерешённые задачи — РЕШЕНО (2 апреля 2026)

- ~~A. Optional auth для рецензий~~ — уже было реализовано (OptionalAuthUser)
- ~~B. Реакции на вопросы~~ — решено ранее
- ~~C. per_page параметр~~ — добавлен в get_reviews и get_questions (1..100, default 20)
- ~~D. Теги на странице книги~~ — get_book теперь возвращает BookWithTags
- ~~E. Удаление статуса чтения~~ — добавлен DELETE /api/books/:id/reading-status
