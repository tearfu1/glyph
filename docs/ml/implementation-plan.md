# План реализации ML-сервиса Glyph

## Текущее состояние
- `ml/` — **не существует**, нужно создать с нуля
- `docker-compose.yml` — уже содержит `ML_SERVICE_URL: http://ml-service:8001` в env бэкенда, но сервисов `qdrant` и `ml-service` нет
- `handlers/ai.rs` — отсутствует в бэкенде
- Фронтенд и бэкенд полностью реализованы

---

## Ответы на ключевые вопросы

**rugpt3small (125M) vs rugpt3medium (350M)?**
→ **rugpt3small**. На Colab free/Pro T4 (16GB VRAM) medium влезет, но обучение будет 3-4x дольше, а выигрыш в качестве для стилизации минимален. Small обучается за 15-30 мин на автора — это позволит быстро итерировать гиперпараметры. Для диплома важнее показать *методологию и сравнения*, чем абсолютное качество.

**Размер чанка 512 vs 256?**
→ **256 токенов** для художественных текстов. Причины:
- Абзацы в литературе короче, чем в технических текстах
- 256 токенов ≈ 150-200 слов ≈ 1-2 абзаца — семантически целостная единица
- Больше чанков → точнее retrieval (меньше шума в каждом чанке)
- Overlap 64 токена (25%) компенсирует разрывы
- В эксперименте сравнишь 256 vs 512, но 256 — дефолт

**Один промпт-шаблон или разные?**
→ **Один базовый шаблон**. LoRA и так переключает стиль. Разные шаблоны — лишняя переменная, усложняющая evaluation. Если останется время, можно добавить вариации как ablation study.

**A/B эксперименты для диплома?**
→ 3 условия (не A/B, а A/B/C):
1. **Baseline** — rugpt3small без LoRA, без RAG (просто вопрос → генерация)
2. **RAG only** — rugpt3small без LoRA + retrieval контекст
3. **RAG + LoRA** — полный pipeline

Это даёт 2 сравнительные таблицы: "эффект RAG" и "эффект LoRA". Для защиты — золото.

---

## Этап 1: Scaffolding (день 1)

### Файлы

**`ml/requirements.txt`**
```
fastapi==0.115.0
uvicorn[standard]==0.30.0
torch==2.2.0
transformers==4.40.0
peft==0.10.0
sentence-transformers==2.7.0
qdrant-client==1.9.0
beautifulsoup4==4.12.3
requests==2.31.0
lxml==5.2.0
nltk==3.8.1
pymorphy3==2.0.2
razdel==0.5.0
numpy==1.26.4
pandas==2.2.0
matplotlib==3.8.0
seaborn==0.13.0
scikit-learn==1.4.0
tqdm==4.66.0
python-dotenv==1.0.1
```

**`ml/Dockerfile`**
```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends gcc && rm -rf /var/lib/apt/lists/*
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY src/ ./src/
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8001"]
```

**`ml/src/main.py`**
```python
from fastapi import FastAPI
from src.api.routes import router
from src.config import settings

app = FastAPI(title="Glyph ML Service")
app.include_router(router, prefix="/api")

@app.get("/health")
def health():
    return {"status": "ok"}
```

**`ml/src/config.py`** — Pydantic Settings: QDRANT_HOST, QDRANT_PORT, MODEL_NAME, EMBEDDING_MODEL, TOP_K, CHUNK_SIZE и т.д.

**`ml/src/api/routes.py`** — заглушка `POST /generate-answer` возвращающая 501.

**Структура каталогов:**
```
ml/
├── Dockerfile
├── requirements.txt
├── .env.example
├── src/
│   ├── __init__.py
│   ├── main.py
│   ├── config.py
│   ├── api/
│   │   ├── __init__.py
│   │   └── routes.py
│   ├── rag/
│   │   ├── __init__.py
│   │   ├── chunker.py
│   │   ├── embedder.py
│   │   └── retriever.py
│   └── generation/
│       ├── __init__.py
│       ├── model.py
│       └── generator.py
├── scripts/
│   ├── download_texts.py
│   ├── preprocess.py
│   ├── build_index.py
│   └── train_lora.py
├── data/
│   ├── raw/
│   ├── processed/
│   └── chunks/
├── models/
│   └── adapters/
└── notebooks/
```

**Обновить `docker-compose.yml`** — добавить сервисы `qdrant` и `ml-service`.

### Критерий готовности
- `docker compose up` поднимает postgres + backend + qdrant + ml-service
- `curl http://localhost:8001/health` → `{"status": "ok"}`
- `curl -X POST http://localhost:8001/api/generate-answer` → 501

---

## Этап 2: Сбор корпусов (день 2-3)

### `ml/scripts/download_texts.py`

Конфигурация авторов (dict):
```python
AUTHORS = {
    "dostoevsky": {
        "id": 1,
        "sources": [
            {"url": "http://az.lib.ru/d/dostoewskij_f_m/text_0060.shtml", "title": "Преступление и наказание"},
            {"url": "http://az.lib.ru/d/dostoewskij_f_m/text_0100.shtml", "title": "Братья Карамазовы"},
            {"url": "http://az.lib.ru/d/dostoewskij_f_m/text_0070.shtml", "title": "Идиот"},
            # ...
        ]
    },
    "chekhov": { ... },
    "bulgakov": { ... },
}
```

Логика:
1. `requests.get(url)` с user-agent и задержкой 2 сек
2. `BeautifulSoup` — извлечение текста из `<dd>` / `<pre>` тегов (формат lib.ru)
3. Для Wikisource — API `action=parse` → HTML → text
4. Сохранение: `data/raw/{author}/{book_title}.txt`
5. Логирование: сколько символов скачано, ошибки

### Критерий готовности
- `data/raw/dostoevsky/` — 4-5 файлов, суммарно ~2.5M символов
- `data/raw/chekhov/` — 30+ файлов, ~1.5M символов
- `data/raw/bulgakov/` — 4-5 файлов, ~1.2M символов
- Скрипт идемпотентный (не перезагружает существующие файлы)

---

## Этап 3: Препроцессинг (день 3-4)

### `ml/scripts/preprocess.py`

Пайплайн очистки:
1. Удаление HTML-артефактов (`&nbsp;`, `&#...;`)
2. Нормализация пробелов, переносов строк
3. Удаление служебных пометок (`[1]`, `*`, заголовков lib.ru)
4. Сохранение: `data/processed/{author}/{book}.txt`

### `ml/src/rag/chunker.py`

```python
def chunk_text(text: str, tokenizer, chunk_size=256, overlap=64) -> list[dict]:
    tokens = tokenizer.encode(text)
    chunks = []
    start = 0
    while start < len(tokens):
        end = min(start + chunk_size, len(tokens))
        chunk_tokens = tokens[start:end]
        chunk_text = tokenizer.decode(chunk_tokens)
        chunks.append({
            "text": chunk_text,
            "start_token": start,
            "end_token": end,
        })
        start += chunk_size - overlap
    return chunks
```

Скрипт `scripts/preprocess.py` вызывает chunker, сохраняет:
`data/chunks/{author}/{book}_chunks.jsonl` — каждая строка: `{"text": "...", "book": "...", "author": "...", "chunk_id": 0}`

### Критерий готовности
- Чанки в JSONL файлах
- Статистика: кол-во чанков на автора (ожидание: Достоевский ~2000, Чехов ~1200, Булгаков ~1000)
- Визуальная проверка: 10 случайных чанков — текст чистый, осмысленный

---

## Этап 4: Индексация в Qdrant (день 4-5)

### `ml/scripts/build_index.py`

```python
from qdrant_client import QdrantClient
from qdrant_client.models import VectorParams, Distance, PointStruct
from sentence_transformers import SentenceTransformer

model = SentenceTransformer("intfloat/multilingual-e5-base")
client = QdrantClient(host="localhost", port=6333)

# Создание коллекции
client.recreate_collection(
    collection_name="glyph_chunks",
    vectors_config=VectorParams(size=768, distance=Distance.COSINE),
)

# Для каждого чанка:
# 1. embedding = model.encode(f"passage: {chunk_text}")  # e5 требует префикс
# 2. PointStruct(id=uuid, vector=embedding, payload={author, book, chapter, text})
# 3. client.upsert(collection_name="glyph_chunks", points=batch)
```

**Важно:** e5-base требует префиксы — `"query: "` для запросов, `"passage: "` для документов.

### `ml/src/rag/embedder.py`
```python
class Embedder:
    def __init__(self, model_name="intfloat/multilingual-e5-base"):
        self.model = SentenceTransformer(model_name)

    def embed_query(self, text: str) -> list[float]:
        return self.model.encode(f"query: {text}").tolist()

    def embed_passage(self, text: str) -> list[float]:
        return self.model.encode(f"passage: {text}").tolist()
```

### `ml/src/rag/retriever.py`
```python
class Retriever:
    def retrieve(self, query: str, author_id: int, top_k: int = 5) -> list[dict]:
        vector = self.embedder.embed_query(query)
        results = self.qdrant.search(
            collection_name="glyph_chunks",
            query_vector=vector,
            query_filter=Filter(must=[
                FieldCondition(key="author", match=MatchValue(value=author_id))
            ]),
            limit=top_k,
        )
        return [{"text": r.payload["text"], "score": r.score, ...} for r in results]
```

### Критерий готовности
- `curl http://localhost:6333/collections/glyph_chunks` — коллекция существует, `points_count` > 0
- Тестовый запрос: `"Почему Раскольников убил старуху?"` → top-5 чанков из Достоевского с score > 0.5
- Фильтрация по автору работает корректно

---

## Этап 5: LoRA fine-tuning (день 5-10)

### `ml/scripts/train_lora.py`

Это **главный скрипт**, запускается на Colab/Kaggle.

```python
from transformers import AutoTokenizer, AutoModelForCausalLM, TrainingArguments, Trainer
from peft import LoraConfig, get_peft_model, TaskType
from datasets import Dataset

# 1. Загрузка базовой модели
model_name = "ai-forever/rugpt3small_based_on_gpt2"
tokenizer = AutoTokenizer.from_pretrained(model_name)
model = AutoModelForCausalLM.from_pretrained(model_name)

# 2. LoRA конфигурация
lora_config = LoraConfig(
    task_type=TaskType.CAUSAL_LM,
    r=8,
    lora_alpha=16,
    lora_dropout=0.05,
    target_modules=["c_attn", "c_proj"],  # GPT-2 attention modules
    bias="none",
)
model = get_peft_model(model, lora_config)
model.print_trainable_parameters()  # ~0.5% от общего числа

# 3. Подготовка данных
# Конкатенация всех текстов автора → токенизация → нарезка на блоки по 512 токенов
# labels = input_ids (авторегрессия)

# 4. Обучение
training_args = TrainingArguments(
    output_dir=f"./models/adapters/{author}",
    num_train_epochs=5,
    per_device_train_batch_size=8,
    learning_rate=2e-4,
    warmup_steps=100,
    logging_steps=50,
    save_strategy="epoch",
    fp16=True,  # на Colab T4
    report_to="none",
)
trainer = Trainer(model=model, args=training_args, train_dataset=dataset)
trainer.train()

# 5. Сохранение только LoRA весов
model.save_pretrained(f"./models/adapters/{author}")
```

### Эксперименты (для диплома)

Сетка гиперпараметров (каждый автор × каждая комбинация):

| Эксперимент | r | alpha | lr | epochs |
|-------------|---|-------|----|--------|
| E1 (default) | 8 | 16 | 2e-4 | 5 |
| E2 (low rank) | 4 | 8 | 2e-4 | 5 |
| E3 (high rank) | 16 | 32 | 2e-4 | 5 |
| E4 (long train) | 8 | 16 | 1e-4 | 10 |

4 эксперимента × 3 автора = 12 прогонов. На Colab T4 ~20-30 мин каждый → ~6-8 часов суммарно.

### Colab ноутбук `notebooks/02_training.ipynb`
1. Mount Google Drive
2. Upload текстов (или скачать прямо в Colab)
3. Цикл по авторам и гиперпараметрам
4. Логирование train_loss, eval_loss по эпохам
5. Скачивание адаптеров → `models/adapters/`

### Критерий готовности
- 3 адаптера в `models/adapters/` (dostoevsky, chekhov, bulgakov), каждый ~5-10 MB
- Training loss графики показывают сходимость
- Таблица 12 экспериментов с финальными loss values

---

## Этап 6: Inference pipeline (день 10-12)

### `ml/src/generation/model.py`

```python
class ModelManager:
    def __init__(self):
        self.tokenizer = AutoTokenizer.from_pretrained(settings.MODEL_NAME)
        self.base_model = AutoModelForCausalLM.from_pretrained(settings.MODEL_NAME)
        self.adapters: dict[str, PeftModel] = {}

    def load_adapter(self, author: str) -> PeftModel:
        if author not in self.adapters:
            adapter_path = f"models/adapters/{author}"
            self.adapters[author] = PeftModel.from_pretrained(
                self.base_model, adapter_path
            )
        return self.adapters[author]
```

### `ml/src/generation/generator.py`

```python
class AnswerGenerator:
    def generate(self, question: str, author: str, context_chunks: list[str]) -> str:
        model = self.model_manager.load_adapter(author)

        context = "\n\n".join(context_chunks)
        prompt = (
            f"Контекст из произведений автора:\n{context}\n\n"
            f"Вопрос читателя: {question}\n\n"
            f"Ответ в стиле автора:"
        )

        inputs = self.tokenizer(prompt, return_tensors="pt", truncation=True, max_length=1024)
        outputs = model.generate(
            **inputs,
            max_new_tokens=256,
            temperature=0.8,
            top_p=0.9,
            repetition_penalty=1.2,
            do_sample=True,
        )
        answer = self.tokenizer.decode(outputs[0][inputs["input_ids"].shape[1]:], skip_special_tokens=True)
        return answer.strip()
```

### Полный pipeline
```
вопрос + author_id
    → Embedder.embed_query(вопрос)
    → Retriever.retrieve(vector, author_id, top_k=5)
    → Generator.generate(вопрос, author, chunks)
    → ответ
```

### Критерий готовности
- Вызов pipeline из Python возвращает осмысленный ответ
- Ответы разных авторов стилистически различаются (визуальная проверка)
- Время генерации: < 30 сек на CPU, < 5 сек на GPU

---

## Этап 7: FastAPI сервис (день 12-13)

### `ml/src/api/routes.py`

```python
@router.post("/generate-answer")
async def generate_answer(request: GenerateRequest) -> GenerateResponse:
    """
    Request:  {"question": str, "author": str, "book_id": int | None}
    Response: {"answer": str, "sources": [{"text": str, "score": float, "book": str}]}
    """
    # 1. Retrieval
    chunks = retriever.retrieve(request.question, request.author, top_k=5)
    # 2. Generation
    answer = generator.generate(request.question, request.author, [c["text"] for c in chunks])
    # 3. Response
    return GenerateResponse(answer=answer, sources=chunks)
```

### Инициализация при старте

В `main.py` — `@app.on_event("startup")`:
- Загрузка Embedder (e5-base) — ~1 GB RAM
- Загрузка base model (rugpt3small) — ~0.5 GB RAM
- Предзагрузка всех 3 адаптеров — ~15 MB
- Подключение к Qdrant

### Критерий готовности
- `docker compose up` — ML-сервис стартует, загружает модели
- `curl -X POST http://localhost:8001/api/generate-answer -H 'Content-Type: application/json' -d '{"question":"Почему Раскольников убил старуху?","author":"dostoevsky"}'` → JSON с ответом и sources

---

## Этап 8: Интеграция с Rust-бэкендом (день 13-15)

### Бэкенд: `backend/src/handlers/ai.rs`

```rust
pub async fn generate_ai_answer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(question_id): Path<i32>,
) -> Result<Json<AiAnswerResponse>, AppError> {
    // 1. Получить вопрос и book_id из БД
    // 2. Получить автора книги
    // 3. HTTP POST к ML_SERVICE_URL/api/generate-answer
    // 4. Сохранить ответ в up_ai_answer
    // 5. Вернуть ответ
}
```

### Маршрут
```rust
// В router:
.route("/api/questions/:id/ai-answer", post(ai::generate_ai_answer))
```

### Миграция БД
```sql
CREATE TABLE up_ai_answer (
    id SERIAL PRIMARY KEY,
    question_id INT NOT NULL REFERENCES up_question(id) ON DELETE CASCADE,
    answer_text TEXT NOT NULL,
    author_style VARCHAR(50) NOT NULL,  -- "dostoevsky", "chekhov", "bulgakov"
    sources JSONB,                       -- [{text, score, book}]
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Фронтенд: `frontend/src/api/ai.ts`
```typescript
export const generateAiAnswer = (questionId: number) =>
    client.post(`/api/questions/${questionId}/ai-answer`)
```

Компонент `AiAnswerCard.vue` — отображение с пометкой "AI-ответ в стиле автора", развёрнутые источники.

### Критерий готовности
- На странице вопроса появляется кнопка "Получить ответ в стиле автора"
- Клик → спиннер → AI-ответ отображается с источниками
- Ответ сохраняется в БД, при перезагрузке страницы виден

---

## Этап 9: Evaluation (день 15-20)

### `ml/src/evaluation/perplexity.py`
```python
def compute_perplexity(model, tokenizer, texts: list[str]) -> float:
    # Стандартный расчёт PPL на held-out set (10% корпуса каждого автора)
```

### `ml/src/evaluation/style_metrics.py`
```python
def compute_style_metrics(text: str) -> dict:
    # pymorphy3 для POS-tagging
    # razdel для сегментации предложений
    return {
        "ttr": unique_words / total_words,
        "avg_sentence_length": total_words / num_sentences,
        "pos_distribution": {"NOUN": 0.25, "VERB": 0.18, ...},
        "hapax_ratio": hapax_count / total_words,
    }
```

### `ml/src/evaluation/semantic_score.py`
```python
def compute_semantic_similarity(answer: str, chunks: list[str], embedder) -> float:
    answer_emb = embedder.embed_query(answer)
    chunk_embs = [embedder.embed_passage(c) for c in chunks]
    scores = [cosine_similarity(answer_emb, ce) for ce in chunk_embs]
    return np.mean(scores)
```

### Сравнительные таблицы для диплома

**Таблица 1: Perplexity**
| Автор | Baseline (no LoRA) | RAG only | RAG + LoRA (r=4) | RAG + LoRA (r=8) | RAG + LoRA (r=16) |
|-------|-------------------|----------|-------------------|-------------------|---------------------|

**Таблица 2: Стилевые метрики (пример для Достоевского)**
| Метрика | Оригинал | Baseline | RAG + LoRA | Δ (LoRA vs Original) |
|---------|----------|----------|------------|----------------------|

**Таблица 3: Semantic Similarity**
| Автор | Baseline | RAG only | RAG + LoRA |
|-------|----------|----------|------------|

### Критерий готовности
- 3 заполненные таблицы с числовыми результатами
- Статистически значимое улучшение RAG+LoRA над baseline хотя бы по perplexity и стилевым метрикам

---

## Этап 10: Jupyter-ноутбуки (день 20-25)

### `notebooks/01_eda.ipynb`
- Статистика корпусов: размер, кол-во слов, уникальных слов
- Распределение длин предложений по авторам (гистограммы)
- Word clouds по авторам
- Топ-50 слов по авторам (bar charts)
- POS-распределение по авторам

### `notebooks/02_training.ipynb`
- Графики loss по эпохам (train + eval) для каждого эксперимента
- Сравнение loss по рангам LoRA
- Время обучения

### `notebooks/03_evaluation.ipynb`
- Все таблицы из этапа 9 в красивом виде
- Bar charts: perplexity по условиям (baseline / RAG / RAG+LoRA)
- Radar charts: стилевые метрики (оригинал vs generated)
- Примеры сгенерированных ответов (по 3 на автора)
- Confusion-подобная матрица: "может ли человек отличить автора от AI"

### Критерий готовности
- Все графики экспортируются в PNG для вставки в диплом
- Ноутбуки запускаются от начала до конца без ошибок

---

## Таймлайн (4 недели)

| Неделя | Дни | Этапы | Приоритет |
|--------|-----|-------|-----------|
| **1** | 1-7 | Scaffolding + Сбор корпусов + Препроцессинг + Индексация (этапы 1-4) | Инфраструктура |
| **2** | 8-14 | LoRA fine-tuning + Inference pipeline + FastAPI (этапы 5-7) | ML-ядро |
| **3** | 15-21 | Интеграция с бэкендом + Evaluation (этапы 8-9) | Продукт + метрики |
| **4** | 22-28 | Jupyter-ноутбуки + графики + доработка + буфер (этап 10) | Диплом |

---

## Обновлённый `docker-compose.yml`

Нужно добавить:
```yaml
  qdrant:
    image: qdrant/qdrant:v1.9.0
    ports:
      - "6333:6333"
    volumes:
      - qdrant_data:/qdrant/storage

  ml-service:
    build: ./ml
    ports:
      - "8001:8001"
    environment:
      QDRANT_HOST: qdrant
      QDRANT_PORT: 6333
      MODEL_PATH: /app/models
    depends_on:
      - qdrant
    volumes:
      - ./ml/models:/app/models
      - ./ml/data:/app/data
```

И `qdrant_data` в `volumes`.
