# Целевая архитектура: Glyph v2

## Общая схема

```
┌─────────────────┐       HTTP/JSON        ┌───────────────────┐
│   Vue 3 SPA     │ ─────────────────────→ │   Rust (Axum)     │
│   TypeScript     │ ←───────────────────── │   REST API        │
│   Tailwind CSS  │                        │   JWT Auth        │
│   Vite          │                        │   SQLx + PgSQL    │
└─────────────────┘                        └────────┬──────────┘
                                                    │ HTTP (internal)
                                             ┌──────▼──────────┐
                                             │  Python ML       │
                                             │  FastAPI          │
                                             │  RAG + LoRA      │
                                             └──────┬──────────┘
                                                    │
                                             ┌──────▼──────────┐
                                             │  Qdrant          │
                                             │  Vector DB       │
                                             └─────────────────┘
```

---

## Бэкенд: Rust (Axum + SQLx + PostgreSQL)

### Стек
| Компонент | Технология | Зачем |
|-----------|-----------|-------|
| Web-framework | **Axum** | Экосистема Tokio, ergonomic API |
| SQL | **SQLx** | Async, compile-time проверка запросов |
| База данных | **PostgreSQL** | Нативный full-text search, лучшие типы |
| Авторизация | **JWT** (access + refresh) | Stateless, подходит для SPA |
| Валидация | **validator** crate | Декларативная валидация struct'ов |
| Сериализация | **serde** | JSON serialization/deserialization |
| Изображения | **image** crate | Ресайз без внешних зависимостей |
| HTTP-клиент | **reqwest** (rustls-tls) | Вызов ML-сервиса |
| Конфигурация | **.env + dotenvy** | Переменные окружения |

### Структура проекта

```
backend/
├── Cargo.toml
├── .env
├── migrations/              # SQLx миграции
│   ├── 001_users.sql
│   ├── 002_books.sql
│   ├── 003_tags.sql
│   ├── 004_reviews.sql
│   ├── 005_questions.sql
│   ├── 006_images.sql
│   ├── 007_reading_status.sql
│   └── 008_ai_answers.sql
├── src/
│   ├── main.rs              # Axum server, router, middleware stack
│   ├── config.rs            # Env config struct
│   ├── errors.rs            # Единая обработка ошибок (AppError)
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── jwt.rs           # Создание/валидация токенов
│   │   ├── middleware.rs     # Auth middleware (замена CheckUserGroupAccess)
│   │   ├── handlers.rs      # login, register, refresh, logout
│   │   └── password.rs      # Argon2 хэширование
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── books.rs         # get_page, get_for_shelf, get_for_author, add_book
│   │   ├── reviews.rs       # CRUD + реакции (8 эндпоинтов)
│   │   ├── questions.rs     # CRUD + реакции (11 эндпоинтов)
│   │   ├── answers.rs       # add_answer
│   │   ├── users.rs         # профиль, настройки, группы (6 эндпоинтов)
│   │   ├── tags.rs          # get_grouped_by_type
│   │   ├── reading_status.rs # list, change
│   │   ├── images.rs        # upload + resize
│   │   └── ai_answers.rs   # генерация и получение AI-ответа в стиле автора
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── book.rs
│   │   ├── review.rs
│   │   ├── question.rs
│   │   ├── answer.rs
│   │   ├── tag.rs
│   │   ├── image.rs
│   │   ├── reading_status.rs
│   │   └── ai_answer.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── book.rs
│   │   ├── review.rs
│   │   ├── question.rs
│   │   ├── answer.rs
│   │   ├── user.rs
│   │   ├── tag.rs
│   │   ├── reading_status.rs
│   │   ├── image.rs
│   │   └── ai_answer.rs    # HTTP-клиент к ML-сервису (reqwest)
│   └── validation/
│       ├── mod.rs
│       ├── book.rs
│       └── user.rs
```

### API-маршруты (REST)

```
POST   /api/auth/login
POST   /api/auth/register
POST   /api/auth/refresh
POST   /api/auth/logout

GET    /api/books?page=&tags[]=&search=
GET    /api/books/:id
POST   /api/books                          # [admin, author]
GET    /api/books/shelf/:userId?status=&page=
GET    /api/books/author/:authorId?page=

GET    /api/books/:bookId/reviews?page=
GET    /api/books/:bookId/reviews/my
POST   /api/books/:bookId/reviews
PUT    /api/reviews/:id
DELETE /api/reviews/:id
POST   /api/reviews/:id/reaction
DELETE /api/reviews/:id/reaction

GET    /api/books/:bookId/questions?page=
GET    /api/books/:bookId/questions/my
GET    /api/books/:bookId/questions/best
POST   /api/books/:bookId/questions         # [premium]
PUT    /api/questions/:id
DELETE /api/questions/:id
POST   /api/questions/:id/reaction
DELETE /api/questions/:id/reaction
GET    /api/questions/incoming?page=         # [author]
GET    /api/questions/answered?page=         # [author]
GET    /api/questions/my?page=

POST   /api/questions/:id/answer             # [author]
GET    /api/questions/:id/ai-answer          # AI-ответ (публичный)
POST   /api/questions/:id/ai-answer          # генерация AI-ответа [author, admin]

GET    /api/tags?skip_types[]=

GET    /api/reading-statuses
POST   /api/books/:bookId/reading-status

GET    /api/users/:id/profile
GET    /api/users/me/settings
PUT    /api/users/me
GET    /api/users/me/navigation
GET    /api/users/:id/reviews?page=
GET    /api/admin/groups                     # [admin]
PUT    /api/admin/users/:id/groups           # [admin]
```

### Авторизация

- Регистрация: email + password → Argon2 хэш → JWT пара
- Access token: в памяти (Pinia), время жизни 15 мин
- Refresh token: httpOnly cookie, время жизни 7 дней
- Middleware проверяет JWT, извлекает user_id + roles
- Роли: `user`, `premium`, `author`, `admin` (enum в PostgreSQL, замена Bitrix-групп)

### База данных — изменения относительно текущей

| Изменение | Причина |
|-----------|---------|
| Убираем `b_user` (Bitrix) → расширяем `up_user` полями login, email, password_hash | Независимость от Bitrix |
| Убираем `up_role`, `up_user_role` | Мёртвый код, заменяем на enum role в users |
| Убираем `up_title_author_search` | PostgreSQL `tsvector` вместо отдельной таблицы |
| Фиксим `up_user_image` → user reference на UserTable | Баг в оригинале |
| Добавляем `up_author_corpus` | Связь автор → корпус текстов для ML |
| Добавляем `up_ai_answer` | AI-сгенерированные ответы: UUID PK, UNIQUE question_id FK, answer_text, sources (JSONB), created_at |

---

## Фронтенд: Vue 3 + TypeScript

### Стек
| Компонент | Технология |
|-----------|-----------|
| Framework | **Vue 3** (Composition API, `<script setup>`) |
| Язык | **TypeScript** |
| Роутинг | **Vue Router 4** |
| Стейт | **Pinia** |
| Стили | **Tailwind CSS** |
| HTTP | **Axios** или **ofetch** |
| Сборка | **Vite** |
| Иконки | **Heroicons** или **Lucide** |

### Структура проекта

```
frontend/
├── index.html
├── vite.config.ts
├── tailwind.config.ts
├── tsconfig.json
├── package.json
├── src/
│   ├── App.vue
│   ├── main.ts
│   ├── router/
│   │   └── index.ts           # 8 маршрутов
│   ├── stores/
│   │   ├── auth.ts            # JWT, текущий юзер, роли
│   │   └── user.ts            # Профиль
│   ├── composables/
│   │   ├── useReactions.ts    # Лайки/дизлайки (вместо 4 копий)
│   │   ├── usePagination.ts   # Пагинация (вместо 6 копий)
│   │   ├── useApi.ts          # Типизированный HTTP-клиент
│   │   └── useAuth.ts         # Хелперы авторизации
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppHeader.vue
│   │   │   ├── AppFooter.vue
│   │   │   └── AppSidebar.vue
│   │   ├── BookCard.vue
│   │   ├── ReviewCard.vue
│   │   ├── QuestionCard.vue
│   │   ├── AiAnswerCard.vue   # NEW: карточка AI-ответа
│   │   ├── ReadingStatus.vue
│   │   ├── Pagination.vue
│   │   ├── StarRating.vue
│   │   ├── ReactionButtons.vue
│   │   └── TagFilter.vue
│   ├── views/
│   │   ├── BookCatalog.vue
│   │   ├── BookDetails.vue
│   │   ├── LoginView.vue
│   │   ├── RegisterView.vue
│   │   ├── UserProfile.vue
│   │   ├── SettingsView.vue
│   │   └── PremiumView.vue
│   ├── api/
│   │   ├── client.ts          # Axios instance + interceptors
│   │   ├── auth.ts
│   │   ├── books.ts
│   │   ├── reviews.ts
│   │   ├── questions.ts
│   │   ├── users.ts
│   │   ├── tags.ts
│   │   └── ai-answers.ts     # GET/POST AI-ответов
│   └── types/
│       ├── book.ts
│       ├── review.ts
│       ├── question.ts
│       ├── user.ts
│       └── ai-answer.ts      # AiAnswer, AiAnswerSource
```

### Маршруты Vue Router

| Path | View | Auth |
|------|------|------|
| `/` | BookCatalog | — |
| `/book/:id` | BookDetails | — |
| `/login` | LoginView | guest only |
| `/register` | RegisterView | guest only |
| `/user/:id` | UserProfile | — |
| `/settings` | SettingsView | required |
| `/premium` | PremiumView | — |

### Ключевые улучшения vs текущий фронт

- **Composables** вместо копипасты: `useReactions()`, `usePagination()`, `useApi()`
- **Единая дизайн-система** (Tailwind) вместо 3 палитр
- **TypeScript** — типизированные API-ответы, модели, пропсы
- **Мобильная адаптивность** — Tailwind responsive utilities
- **AiAnswerCard** — новый компонент для AI-ответов с визуальной пометкой "AI"

---

## Инфраструктура

### Docker Compose (dev)

```yaml
services:
  postgres:
    image: postgres:16
    ports: ["5432:5432"]
    volumes: [pgdata:/var/lib/postgresql/data]

  qdrant:
    image: qdrant/qdrant
    ports: ["6333:6333"]
    volumes: [qdrant_data:/qdrant/storage]

  ml-service:
    build: ./ml
    ports: ["8001:8001"]
    depends_on: [qdrant]
    volumes: [./ml/models:/app/models]
```

Rust-бэкенд и Vue-фронтенд запускаются нативно при разработке (`cargo run`, `npm run dev`).

### Деплой (MVP)

Один VPS с Docker Compose — достаточно для защиты диплома.
