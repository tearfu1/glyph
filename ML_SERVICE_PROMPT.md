# Промпт для следующей сессии: ML-сервис Glyph

Скопируй этот текст как первое сообщение в новую сессию Claude Code.

---

Проект Glyph — платформа для книголюбов. Дипломный проект, защита: конец мая — июнь 2026.

Архитектура описана в docs/architecture/target-state.md. ML-подход детально описан в docs/ml/approach.md — обязательно прочитай оба файла перед началом.

## Контекст

Бэкенд (Rust/Axum) и фронтенд (Vue 3) полностью реализованы. Сейчас нужно спроектировать и реализовать **ML-сервис** — ключевую часть диплома.

**Суть:** пользователь задаёт вопрос по книге, система генерирует ответ **в стиле автора** (Достоевский, Чехов, Булгаков), используя RAG + LoRA fine-tuning.

## Стек ML-сервиса (уже определён в docs/ml/approach.md)

- Python + FastAPI
- PyTorch + HuggingFace (PEFT для LoRA, Transformers)
- Базовая модель: `ai-forever/rugpt3small_based_on_gpt2` (~125M params)
- Эмбеддинги: `intfloat/multilingual-e5-base` (768 dims)
- Vector DB: Qdrant (Docker)
- Тексты: public domain (lib.ru, Wikisource)

## Что нужно спланировать

Составь детальный пошаговый план реализации ML-сервиса. Для каждого шага укажи:
- Конкретные файлы и их содержимое
- Команды для запуска
- Ожидаемый результат / критерий готовности

### Этапы для планирования:

1. **Scaffolding** — структура `ml/`, Dockerfile, requirements.txt, FastAPI app
2. **Сбор корпусов** — скрипт скачивания текстов с lib.ru / Wikisource, очистка от HTML/разметки
3. **Препроцессинг** — нормализация, tokenization, chunking (512 tokens, 64 overlap), сохранение с метаданными
4. **Индексация в Qdrant** — эмбеддинги чанков через e5-base, загрузка в Qdrant с payload (author_id, book_id, chapter, text)
5. **LoRA fine-tuning** — скрипт обучения адаптера на корпусе каждого автора, подбор гиперпараметров (rank, alpha, lr, epochs)
6. **Inference pipeline** — вопрос → embedding → Qdrant retrieval (top-k) → промпт с контекстом → rugpt3+LoRA → ответ
7. **FastAPI сервис** — `POST /generate-answer` endpoint, загрузка моделей, переключение адаптеров по author_id
8. **Интеграция с Rust-бэкендом** — эндпоинт `POST /api/questions/:id/ai-answer` проксирует запрос в ML-сервис
9. **Evaluation** — perplexity, стилевые метрики (TTR, средняя длина предложений, POS), semantic similarity, human evaluation
10. **Jupyter-ноутбуки** — EDA корпусов, визуализация обучения, сравнительные таблицы/графики для диплома

### Ограничения:
- GPU: Google Colab Pro / Kaggle (бесплатный GPU). Локально только CPU
- Время: ~4 недели на всё (ML — приоритет №1)
- На защите нужны: формулы, графики, сравнительные таблицы, демо
- Docker Compose: ml-service + qdrant должны добавиться к существующим postgres + backend

### Вопросы для обсуждения:
- rugpt3small (125M) vs rugpt3medium (350M) — что реалистичнее для Colab?
- Размер чанка 512 vs 256 — что лучше для RAG по художественным текстам?
- Один промпт-шаблон или разные для разных типов вопросов?
- Как организовать A/B эксперименты (LoRA vs baseline) для дипломной работы?

Начни с чтения docs/ml/approach.md и docs/architecture/target-state.md, затем предложи план.
