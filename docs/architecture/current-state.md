# Текущее состояние проекта (Bitrix Framework)

## Обзор

Книжная социальная платформа **"Glyph"** (модуль `up.glyph`), написанная на Bitrix Framework.
Аналог Goodreads в миниатюре: каталог книг, рецензии, вопросы авторам, книжные полки, профили.

Репозиторий: https://github.com/tearfu1/finalproject

---

## Функционал

- Каталог книг с фильтрацией по тегам и полнотекстовым поиском
- Рецензии (1-5 звёзд) с лайками/дизлайками
- Вопросы авторам (премиум-фича) с ответами и реакциями
- Книжные полки (планирую / читаю / прочитал)
- Публичные профили пользователей (аватарки, bio)
- Админка — управление группами, добавление книг
- Роли: обычный юзер, премиум (группа 8), верифицированный автор (группа 6), админ (группа 1)

---

## Бэкенд

### Контроллеры (`lib/Controller/*.php`)

Все контроллеры наследуют `Bitrix\Main\Engine\Controller`, используют DI через `ServiceLocator`.

#### Auth Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `loginAction` | login, password, remember | Аутентификация |
| `registerAction` | login, name, lastName, password, confirmPassword, email | Регистрация |

#### Book Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `getPageAction` | pageNumber, tagsByTagTypes[], search | Каталог с фильтрами и поиском (8/стр) |
| `getPageForUserBookshelfAction` | pageUserId, shelfCode, pageNumber | Книги с полки юзера |
| `getPageForAuthorAction` | authorId, pageNumber | Книги автора |
| `addBookAction` | title, description, publisher, publicationDate, isbn, language, number, genres[], file | Добавление книги (группы 1,6) |

#### User Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `getProfilePublicInfoAction` | pageUserId | Публичный профиль |
| `getSettingsProfileDataAction` | — | Данные профиля для редактирования |
| `getNavigationBarAction` | — | Меню навигации по ролям |
| `getAllGroupsAction` | — | Все группы (только админ) |
| `updateUserAction` | name, last-name, self-info, file | Обновление профиля |
| `updateUserGroupAction` | userId, groups[] | Смена группы (только админ) |

#### Question Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `addQuestionAction` | bookId, content | Задать вопрос (1 на книгу, группа 8) |
| `updateQuestionAction` | fields{questionId, content} | Редактирование (до 1 часа) |
| `deleteQuestionAction` | fields{questionId} | Soft-delete (до 1 часа) |
| `getUserQuestionsByBookIdAction` | bookId | Мои вопросы по книге |
| `getQuestionsByBookIdAction` | bookId, page | Все вопросы по книге (5/стр) |
| `getAuthorUnansweredQuestionsAction` | page | Входящие без ответа (для автора) |
| `getAuthorAnsweredQuestionsAction` | page | Отвеченные (для автора) |
| `getUserQuestionsPageAction` | page | Мои вопросы |
| `getBestQuestionByBookIdAction` | bookId | Лучший вопрос (тизер для не-премиум) |
| `addQuestionUserReactionAction` | questionId, isLikeValue | Лайк/дизлайк |
| `deleteQuestionUserReactionAction` | questionId | Убрать реакцию |

#### Answer Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `addAnswerAction` | questionId, content | Ответ автора (1 на вопрос, группа 6) |

#### Review Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `getBookReviewsAction` | page, bookId | Рецензии на книгу (5/стр) |
| `getUserReviewByBookIdAction` | bookId | Моя рецензия |
| `getUserReviewsPageAction` | page, pageUserId | Рецензии юзера |
| `addReviewAction` | bookId, title, content, rate | Добавить рецензию (1-5, 1 на книгу) |
| `updateReviewAction` | fields{reviewId, title, content, rate} | Редактирование |
| `deleteReviewAction` | fields{reviewId} | Soft-delete |
| `addUserReactionToReviewAction` | reviewId, isLikeValue | Лайк/дизлайк |
| `deleteUserReactionToReviewAction` | reviewId | Убрать реакцию |

#### ReadingStatus Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `getReadingStatusesListAction` | — | Список статусов |
| `changeBookReadingStatusAction` | bookId, readingStatusId | Переключить статус |

#### Tag Controller
| Экшен | Параметры | Назначение |
|-------|-----------|------------|
| `getListGroupedByTypeAction` | skipTagTypeIds[] | Теги по типам |

### Сервисы (`lib/Service/*.php`)

| Сервис | Ключевая логика |
|--------|----------------|
| **Auth** | Регистрация через `CUser::Register`, создание `up_user`, авто-авторизация |
| **Book** | Сложная пагинация с AND-фильтрацией тегов, FULLTEXT поиск, обработка изображений (3 размера) |
| **Review** | CRUD + пересчёт рейтинга книги (AVG/COUNT), реакции с блокировками |
| **Question** | CRUD с лимитом редактирования 1 час, 4 режима фильтрации, реакции с блокировками |
| **Answer** | Валидация: вопрос существует, нет ответа, юзер — автор книги |
| **User** | Профиль, навигация по ролям, смена групп (при добавлении автора → персональный тег) |
| **ReadingStatus** | Toggle-логика: повтор статуса удаляет, другой — обновляет |
| **Tag** | Теги сгруппированные по типам |
| **Image** | Валидация загрузки (макс 10MB), ресайз в 3 размера: small(300), medium(800), large(1200) |

### Модели (БД) — 17 таблиц

#### Основные таблицы
| Таблица | Поля |
|---------|------|
| `up_book` | id, title, description, rating(3,2), publisher, publication_date, isbn, language, page_number, review_number, created_at, updated_at, deleted_at |
| `up_user` | id (FK→b_user), self_info, personal_tag_id, timestamps |
| `up_review` | id, user_id, book_id, title, content, rate(1-5), like_number, dislike_number, timestamps |
| `up_question` | id, user_id, book_id, content, like_number, dislike_number, timestamps |
| `up_answer` | id, user_id, question_id, content, like_number, dislike_number, timestamps |
| `up_tag` | id, code(unique), name, type_id, timestamps |
| `up_tag_type` | id, name(unique), timestamps |
| `up_image` | id, image_path, width, height, size_alias_id, is_main, timestamps |
| `up_image_size_alias` | id, name, timestamps |
| `up_book_reading_status` | id, name, timestamps |
| `up_role` | id, name, timestamps — **НЕ ИСПОЛЬЗУЕТСЯ** |
| `up_title_author_search` | book_id(PK), book_title, author_name — FULLTEXT индекс |

#### Связующие таблицы
| Таблица | Связь |
|---------|-------|
| `up_author_book` | user_id + book_id (автор↔книга) |
| `up_book_tag` | book_id + tag_id |
| `up_book_image` | book_id + image_id |
| `up_user_image` | user_id + image_id |
| `up_user_book` | user_id + book_id + reading_status_id (полки) |
| `up_user_role` | user_id + role_id + expiration_time — **НЕ ИСПОЛЬЗУЕТСЯ** |

#### Таблицы реакций
| Таблица | Связь |
|---------|-------|
| `up_review_reaction` | user_id + review_id + is_like |
| `up_question_reaction` | user_id + question_id + is_like |
| `up_answer_reaction` | user_id + answer_id + is_like |

### Валидация

**BookFieldsValidator:** title(req, 100), description(req, 5000), publisher(100), publicationDate(Y-m-d\TH:i:s), isbn(10/13 цифр), number(positive int), language(50), genres(array)

**UserFieldsValidator:** name(req, 50), last-name(req, 50), self-info(250)

### Фильтр авторизации

`CheckUserGroupAccess` — проверяет принадлежность к группам:
- `[1]` = admin
- `[6]` = verified author
- `[8]` = premium
- `[1, 6]` = admin или author
- `[6, 8]` = author или premium

### Маршруты

| URL | View | Назначение |
|-----|------|------------|
| `/` | main-page.php | Каталог |
| `/book/{id}/` | book-details.php | Детали книги |
| `/login` | login-page.php | Вход |
| `/register` | register-page.php | Регистрация |
| `/logout` | logout-page.php | Выход |
| `/user/{id}/` | user-public-page.php | Профиль |
| `/settings/` | settings-page.php | Настройки |
| `/premium/` | premium-page.php | Премиум |

AJAX: автороутинг через `/bitrix/services/main/ajax.php?action=up:glyph.api.{controller}.{action}`

### Известные баги
- `UserImageTable.php:58` — ссылка на `BookTable` вместо `UserTable`
- `profile-review-list/bundle.config.js:4` — namespace `'BX.'` вместо `'BX.Up.Glyph'`
- `RoleTable` и `UserRoleTable` — мёртвый код

---

## Фронтенд

### JS-модули (12 штук, vanilla JS через Bitrix `main.core`)

| Модуль | Назначение | API-вызовы |
|--------|-----------|------------|
| `up.auth` | Формы логина и регистрации, валидация | auth.login, auth.register |
| `up.book-list` | Каталог: сайдбар фильтров, грид карточек, пагинация | book.getPage, tag.getListGroupedByType |
| `up.book-details` | Моя рецензия, табы (Рецензии/Вопросы) | review.getUserReviewByBookId, review.addReview, реакции |
| `up.review-list` | Все рецензии книги, пагинация | review.getBookReviews, реакции |
| `up.question-list` | Вопросы автору, премиум-гейтинг, тизер | question.*, reactions |
| `up.reading-status` | Дропдаун статуса чтения (2 режима: detail/index) | readingstatus.* |
| `up.pagination` | Переиспользуемый компонент пагинации | — (callbacks) |
| `up.profile-public` | Оркестратор профиля: header + табы | user.getProfilePublicInfo |
| `up.profile-book-list` | Книжные полки юзера | book.getPageForUserBookshelf |
| `up.profile-author-book-list` | Книги автора | book.getPageForAuthor |
| `up.profile-review-list` | Рецензии юзера | review.getUserReviewsPage, реакции |
| `up.settings-profile` | Настройки: 6 вкладок (аккаунт, добавление книги, вопросы, админ) | user.*, tag.*, book.addBook, question.*, answer.addAnswer |

### Проблемы фронтенда
- Реакции (лайк/дизлайк) скопированы **4 раза** (book-details, review-list, profile-review-list, question-list)
- Пагинация дублируется **6+ раз**
- Форматирование дат дублируется **5 раз**
- **3 разные CSS-палитры**: warm (каталог), neutral (профиль), settings
- ~600 строк дублированного CSS
- `min-width: 1100px` ломает мобильную адаптивность

### Полная карта API-эндпоинтов (31 штука)

```
auth.login, auth.register
book.getPage, book.getPageForUserBookshelf, book.getPageForAuthor, book.addBook
tag.getListGroupedByType
review.getUserReviewByBookId, review.addReview, review.getBookReviews,
  review.getUserReviewsPage, review.deleteUserReactionToReview, review.addUserReactionToReview
question.getUserQuestionsByBookId, question.getQuestionsByBookId,
  question.getBestQuestionByBookId, question.addQuestion,
  question.deleteQuestionUserReaction, question.addQuestionUserReaction,
  question.getAuthorUnansweredQuestions, question.getAuthorAnsweredQuestions,
  question.getUserQuestionsPage
answer.addAnswer
readingstatus.getReadingStatusesList, readingstatus.changeBookReadingStatus
user.getProfilePublicInfo, user.getSettingsProfileData, user.getNavigationBar,
  user.updateUser, user.getAllGroups, user.updateUserGroup
```
