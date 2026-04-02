# Страница книги (BookDetails) — PRD

## Статус: в разработке

---

## Что реализовано

### 1. Основная информация о книге

**Файлы:** `frontend/src/views/BookDetails.vue`, `frontend/src/api/books.ts`

- Загрузка данных через `GET /api/books/:id` → `BookWithAuthor`
- Обложка (aspect 2:3) с плейсхолдером-иконкой
- Заголовок, автор (аватар-кружок + display_name), год, ISBN
- Описание с `whitespace-pre-line`
- Теги с цветовой кодировкой: genre=indigo, mood=amber, theme=emerald, period=slate
- Ссылка «← Каталог» наверху
- Спиннер при загрузке, заглушка «Книга не найдена»

### 2. Статус чтения

**Файлы:**
- Бэкенд: `handlers/reading_status.rs`, `services/reading_status.rs`, `main.rs`
- Фронтенд: `api/reading-status.ts`, `types/reading-status.ts`, `views/BookDetails.vue`, `views/HomeView.vue`, `components/BookCard.vue`

**Эндпоинты:**
- `POST /api/books/:id/reading-status` — установить статус `{ status: "want_to_read" | "reading" | "read" }` (уже был)
- `GET /api/reading-statuses/my` — **новый**, возвращает `Vec<ReadingStatus>` текущего пользователя (требует авторизации)

**Поведение:**
- На странице книги: dropdown-кнопка (Хочу прочитать / Читаю / Прочитано), виден только авторизованным
- В каталоге: статус отображается на каждой карточке BookCard; загружается одним запросом `getMyStatuses()` для всех книг
- Смена статуса из каталога через dropdown без перехода на страницу
- Оптимистичное обновление UI (сначала меняется map, потом уходит запрос)
- Цвета: want_to_read=amber, reading=indigo, read=emerald

### 3. Рецензии

**Файлы:** `api/reviews.ts`, `types/review.ts`, `views/BookDetails.vue`

**Эндпоинты (все уже были на бэкенде):**
- `GET /api/books/:id/reviews?page=N` → `PaginatedResponse<ReviewWithUser>`
- `GET /api/books/:id/reviews/my` → `ReviewWithUser | null` (требует авторизации)
- `POST /api/books/:id/reviews` → `Review` (rating 1-5, text)
- `PUT /api/reviews/:id` → `Review`
- `DELETE /api/reviews/:id`
- `POST /api/reviews/:id/reaction` → `ReviewReaction` (is_like: bool)
- `DELETE /api/reviews/:id/reaction`

**Поведение:**
- Табы «Рецензии» / «Вопросы автору» с бейджами-счётчиками
- Форма написания рецензии показывается сразу (если авторизован и нет рецензии)
- Рейтинг звёздами — интерактивный выбор 1-5
- Своя рецензия — отдельный выделенный блок сверху с кнопками «Редактировать» / «Удалить»
- Своя рецензия отфильтрована из общего списка по `user_id`
- Каждая рецензия: аватар, имя, звёзды, текст, дата, кнопки like/dislike
- Для неавторизованных: промпт «Войдите, чтобы оставить рецензию»

### 4. Вопросы автору

**Файлы:** `api/questions.ts`, `types/question.ts`, `views/BookDetails.vue`

**Бэкенд-изменения:**
- `models/question.rs` — `QuestionWithUser` расширен: `answer_text`, `answer_created_at`, `answer_user_display_name`, `answer_user_avatar_url` (все `Option`)
- `services/question.rs` — SQL расширен `LEFT JOIN up_answer a ON a.question_id = q.id` + `LEFT JOIN up_user au ON au.id = a.user_id`

**Эндпоинты:**
- `GET /api/books/:id/questions?page=N` → `PaginatedResponse<QuestionWithUser>`
- `POST /api/books/:id/questions` → `Question` (только premium/admin)
- `POST /api/questions/:id/answer` → `Answer` (только author/admin книги)

**Поведение:**
- Форма вопроса видна сразу для premium/admin
- Список: аватар, имя, текст, дата, бейдж «Есть ответ», счётчик лайков
- Ответ отображается под вопросом: indigo-полоска слева, аватар + имя автора, бейдж «Автор», дата, текст

### 5. Созданные файлы

```
frontend/src/types/review.ts          — Review, ReviewWithUser
frontend/src/types/question.ts        — Question, QuestionWithUser (с полями ответа)
frontend/src/types/reading-status.ts  — ReadingStatusType, ReadingStatus
frontend/src/api/reviews.ts           — getReviews, getMyReview, createReview, updateReview, deleteReview, addReaction, removeReaction
frontend/src/api/questions.ts         — getQuestions, createQuestion
frontend/src/api/reading-status.ts    — getMyStatuses, setReadingStatus
```

---

## Что НЕ реализовано (TODO)

### A. Реакции на рецензии — не работают полноценно

**Проблема:** `handlers/reviews.rs` → `get_reviews()` хардкодит `current_user_id: Option<Uuid> = None`. Сервис уже поддерживает `Some(uid)` — при передаче user_id SQL возвращает `user_reaction` (какую реакцию поставил пользователь).

**Что нужно:**
1. В `handlers/reviews.rs` сделать авторизацию опциональной: `Option<AuthUser>` работает в Axum (если `FromRequestParts` возвращает ошибку, `Option` ловит `None`)
2. Передать `auth.map(|a| a.0.sub)` в `review_service::get_reviews`
3. На фронте уже всё готово — `toggleReaction` и подсветка `user_reaction` реализованы

### ~~B. Реакции на вопросы — не интерактивны~~ РЕШЕНО

Реакции на вопросы подключены через `useReactions` composable + `ReactionButtons` компонент.
API `addReaction` / `removeReaction` уже были в `questions.ts`. `user_reaction` уже есть в `QuestionWithUser`.

### C. Пагинация — бэкенд возвращает по 20

**Проблема:** Фронтенд инициализирует `perPage = 10`, но бэкенд хардкодит `PAGE_SIZE = 20` и возвращает `per_page: 20` в ответе. Фронт перезаписывает значение из ответа.

**Что нужно:**
1. Добавить query-параметр `per_page` в бэкенд-хэндлеры `get_reviews` и `get_questions`
2. Фронтенд уже отправляет `page`, нужно добавить отправку `per_page=10`

### D. Теги на странице книги

**Проблема:** `GET /api/books/:id` не включает теги (они есть только в списке `GET /api/books`).

**Что нужно:**
1. В бэкенде `services/book.rs` → запрос `get_book` дополнить JOIN/подзапросом для тегов
2. Или добавить отдельный endpoint `GET /api/books/:id/tags`

### E. Удаление статуса чтения

Сейчас можно только установить/сменить статус, но не убрать его. Нужен `DELETE /api/books/:id/reading-status`.
