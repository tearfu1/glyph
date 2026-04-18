"""Генерация synthetic instruction dataset для LoRA-обучения.

Для каждого выбранного чанка из корпуса автора LLM генерирует пару
(вопрос читателя, ответ в стиле автора). Результат сохраняется в
data/synthetic/{author}.jsonl для последующего instruction-tuning.

Провайдеры LLM (задаётся --provider или env LLM_PROVIDER):
  - groq      (бесплатно, default; llama-3.3-70b-versatile)
  - gemini    (бесплатно; gemini-2.0-flash)
  - anthropic (платно; claude-haiku-4-5)

API ключ читается из env:
  GROQ_API_KEY / GEMINI_API_KEY / ANTHROPIC_API_KEY

Скрипт идемпотентный: повторный запуск продолжает с места остановки,
читая уже обработанные chunk_id из выходного файла.

Использование:
    python -m scripts.generate_synthetic_dataset --author dostoevsky --num-samples 500
    python -m scripts.generate_synthetic_dataset --author chekhov --provider gemini
"""

from __future__ import annotations

import argparse
import json
import logging
import os
import random
import re
import sys
import time
from pathlib import Path
from typing import Callable

import httpx

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

BASE_DIR = Path(__file__).resolve().parent.parent
PROCESSED_DIR = BASE_DIR / "data" / "processed"
OUTPUT_DIR = BASE_DIR / "data" / "synthetic"

# Размер нарезки processed-текста на куски для подачи в LLM.
# Не совпадает с чанками для RAG (256 токенов) — здесь нужны более крупные
# семантически целостные фрагменты для осмысленных Q&A пар.
CHUNK_CHAR_SIZE = 1000

AUTHOR_NAMES_RU = {
    "dostoevsky": "Фёдор Михайлович Достоевский",
    "chekhov": "Антон Павлович Чехов",
    "bulgakov": "Михаил Афанасьевич Булгаков",
}

SYSTEM_PROMPT = (
    "Ты готовишь датасет для обучения языковой модели отвечать на вопросы читателей "
    "в стиле русского писателя-классика. Возвращай ответы строго в формате JSON."
)

USER_PROMPT_TEMPLATE = """Автор: {author_ru}

Фрагмент произведения:
---
{chunk_text}
---

Задача:
1. Придумай 1 вопрос читателя, на который этот фрагмент даёт ответ. Вопрос о содержании \
(идеи, персонажи, мотивы, атмосфера), не дословный, не про грамматику. Формулируй \
естественно — как живой вопрос в книжном клубе.
2. Сформулируй ответ в стиле автора (2-4 предложения). Звучи как размышление самого автора \
над текстом, а не как пересказ. Опирайся только на содержание фрагмента; не придумывай \
внешних фактов.

Верни строго JSON без пояснений и комментариев:
{{"question": "<текст вопроса>", "answer": "<текст ответа>"}}"""


# ---------- LLM providers ----------

class RateLimitError(Exception):
    """HTTP 429 — нужно подождать и попробовать снова."""


def call_groq(messages: list[dict], model: str = "llama-3.1-8b-instant") -> str:
    api_key = os.getenv("GROQ_API_KEY")
    if not api_key:
        raise RuntimeError("GROQ_API_KEY не задан")
    r = httpx.post(
        "https://api.groq.com/openai/v1/chat/completions",
        headers={"Authorization": f"Bearer {api_key}"},
        json={
            "model": model,
            "messages": messages,
            "temperature": 0.7,
            "response_format": {"type": "json_object"},
        },
        timeout=60.0,
    )
    if r.status_code == 429:
        # Groq передаёт retry-after в заголовках; берём оттуда или дефолт 30 сек
        wait = float(r.headers.get("retry-after", 30))
        raise RateLimitError(f"HTTP 429, retry after {wait}s")
    r.raise_for_status()
    return r.json()["choices"][0]["message"]["content"]


def call_gemini(messages: list[dict], model: str = "gemini-2.0-flash") -> str:
    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key:
        raise RuntimeError("GEMINI_API_KEY не задан")
    # Склеиваем system + user в единый user-prompt, т.к. Gemini REST API так проще
    combined = "\n\n".join(m["content"] for m in messages)
    r = httpx.post(
        f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent",
        params={"key": api_key},
        json={
            "contents": [{"parts": [{"text": combined}]}],
            "generationConfig": {
                "temperature": 0.7,
                "responseMimeType": "application/json",
            },
        },
        timeout=60.0,
    )
    r.raise_for_status()
    data = r.json()
    return data["candidates"][0]["content"]["parts"][0]["text"]


def call_anthropic(messages: list[dict], model: str = "claude-haiku-4-5") -> str:
    api_key = os.getenv("ANTHROPIC_API_KEY")
    if not api_key:
        raise RuntimeError("ANTHROPIC_API_KEY не задан")
    # Anthropic API использует отдельное поле system
    system = "\n".join(m["content"] for m in messages if m["role"] == "system")
    user_msgs = [m for m in messages if m["role"] != "system"]
    r = httpx.post(
        "https://api.anthropic.com/v1/messages",
        headers={
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        },
        json={
            "model": model,
            "max_tokens": 1024,
            "system": system,
            "messages": user_msgs,
            "temperature": 0.7,
        },
        timeout=60.0,
    )
    r.raise_for_status()
    return r.json()["content"][0]["text"]


PROVIDERS: dict[str, Callable[[list[dict], str], str]] = {
    "groq": call_groq,
    "gemini": call_gemini,
    "anthropic": call_anthropic,
}

DEFAULT_MODELS = {
    # llama-3.1-8b-instant имеет выше TPM (30k) чем 70b (12k) — для training data хватает
    "groq": "llama-3.1-8b-instant",
    "gemini": "gemini-2.0-flash",
    "anthropic": "claude-haiku-4-5",
}


# ---------- Core logic ----------

def load_chunks(author: str) -> list[dict]:
    """Нарезает processed-тексты автора на крупные куски по ~1500 символов.

    Нарезка по границам абзацев (\\n\\n), с резервом — по точке. Возвращает
    список dict-ов, совместимых по схеме с build_index чанками:
    {text, book, chunk_id, author}.
    """
    author_dir = PROCESSED_DIR / author
    if not author_dir.exists():
        raise FileNotFoundError(f"Нет processed-текстов автора: {author_dir}")

    chunks: list[dict] = []
    for txt_path in sorted(author_dir.glob("*.txt")):
        book = txt_path.stem
        text = txt_path.read_text(encoding="utf-8")
        # Делим по двойным переносам (абзацы), группируем до CHUNK_CHAR_SIZE
        paragraphs = [p.strip() for p in text.split("\n\n") if p.strip()]
        buf = ""
        idx = 0
        for p in paragraphs:
            if len(buf) + len(p) + 2 > CHUNK_CHAR_SIZE and buf:
                chunks.append({"text": buf, "book": book, "chunk_id": idx, "author": author})
                idx += 1
                buf = p
            else:
                buf = f"{buf}\n\n{p}" if buf else p
        if buf:
            chunks.append({"text": buf, "book": book, "chunk_id": idx, "author": author})

    logger.info("Автор %s: нарезано %d кусков (по ~%d символов)",
                author, len(chunks), CHUNK_CHAR_SIZE)
    return chunks


def load_processed_ids(output_path: Path) -> set[str]:
    """Читает уже обработанные chunk-id из существующего файла (для resume)."""
    if not output_path.exists():
        return set()
    ids: set[str] = set()
    with open(output_path, "r", encoding="utf-8") as f:
        for line in f:
            try:
                rec = json.loads(line)
                ids.add(f"{rec['book']}__{rec['chunk_id']}")
            except Exception:
                continue
    return ids


def parse_llm_json(text: str) -> tuple[str, str] | None:
    """Из ответа LLM выдёргивает {question, answer}."""
    # Пробуем напрямую
    try:
        data = json.loads(text)
        return data["question"], data["answer"]
    except (json.JSONDecodeError, KeyError, TypeError):
        pass

    # Пробуем вырезать JSON из markdown-блока
    m = re.search(r"\{[^{}]*\"question\"[^{}]*\"answer\"[^{}]*\}", text, re.DOTALL)
    if m:
        try:
            data = json.loads(m.group(0))
            return data["question"], data["answer"]
        except (json.JSONDecodeError, KeyError):
            pass

    return None


def generate_pair(
    chunk: dict,
    author: str,
    provider_fn: Callable,
    model: str,
    max_rate_limit_retries: int = 5,
) -> dict | None:
    """Один вызов LLM → распарсенная пара. None при ошибке/пропуске.

    Обрабатывает 429 через ожидание (exponential backoff).
    Другие ошибки — пропуск с логированием.
    """
    author_ru = AUTHOR_NAMES_RU.get(author, author)
    user_prompt = USER_PROMPT_TEMPLATE.format(
        author_ru=author_ru,
        chunk_text=chunk["text"],
    )
    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": user_prompt},
    ]

    raw = None
    for attempt in range(max_rate_limit_retries):
        try:
            raw = provider_fn(messages, model)
            break
        except RateLimitError as e:
            wait = 30 * (2 ** attempt)  # 30, 60, 120, 240, 480 sec
            wait = min(wait, 120)
            logger.info("Rate limit hit, ждём %d сек (попытка %d/%d)...",
                        wait, attempt + 1, max_rate_limit_retries)
            time.sleep(wait)
        except Exception as e:
            logger.warning("LLM call failed (chunk %s/%s): %s",
                           chunk.get("book"), chunk.get("chunk_id"), e)
            return None

    if raw is None:
        logger.warning("Все попытки rate-limit retry исчерпаны для %s/%s",
                       chunk.get("book"), chunk.get("chunk_id"))
        return None

    parsed = parse_llm_json(raw)
    if not parsed:
        logger.warning("JSON parse failed for %s/%s",
                       chunk.get("book"), chunk.get("chunk_id"))
        return None

    question, answer = parsed
    return {
        "author": author,
        "book": chunk["book"],
        "chunk_id": chunk["chunk_id"],
        "context": chunk["text"],
        "question": question.strip(),
        "answer": answer.strip(),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--author", required=True, choices=list(AUTHOR_NAMES_RU))
    parser.add_argument("--num-samples", type=int, default=500)
    parser.add_argument("--provider", choices=list(PROVIDERS),
                        default=os.getenv("LLM_PROVIDER", "groq"))
    parser.add_argument("--model", default=None,
                        help="override default model for provider")
    parser.add_argument("--min-chunk-length", type=int, default=200,
                        help="пропускать чанки короче N символов (служебные куски)")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--rpm", type=int, default=12,
                        help="rate limit — запросов в минуту (default 12 для Groq free)")
    args = parser.parse_args()

    provider_fn = PROVIDERS[args.provider]
    model = args.model or DEFAULT_MODELS[args.provider]
    logger.info("Provider: %s, model: %s", args.provider, model)

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    output_path = OUTPUT_DIR / f"{args.author}.jsonl"

    chunks = load_chunks(args.author)
    chunks = [c for c in chunks if len(c["text"]) >= args.min_chunk_length]
    logger.info("После фильтра по длине (≥%d симв): %d чанков",
                args.min_chunk_length, len(chunks))

    # Resume: пропускаем уже обработанные
    done_ids = load_processed_ids(output_path)
    if done_ids:
        logger.info("Уже обработано: %d пар (resume)", len(done_ids))
    chunks = [c for c in chunks if f"{c['book']}__{c['chunk_id']}" not in done_ids]

    # Воспроизводимая выборка
    random.seed(args.seed)
    random.shuffle(chunks)
    need = max(0, args.num_samples - len(done_ids))
    chunks = chunks[:need]

    if not chunks:
        logger.info("Нечего генерировать — датасет уже готов")
        return

    logger.info("Нужно сгенерировать: %d пар", len(chunks))

    # Smoke-тест: первый запрос ошибается → сразу падаем (auth/quota проблемы)
    logger.info("Проверка API ключа одним тестовым запросом...")
    probe = generate_pair(chunks[0], args.author, provider_fn, model)
    if probe is None:
        logger.error("Тестовый запрос провалился. Проверь API ключ / квоту провайдера.")
        sys.exit(1)
    logger.info("API работает. Пример: Q=%r  A=%r", probe["question"][:80], probe["answer"][:100])

    sleep_between = max(60.0 / args.rpm, 0.0)
    written = 0
    consecutive_fails = 0
    t_start = time.time()

    with open(output_path, "a", encoding="utf-8") as out:
        # Записываем probe
        out.write(json.dumps(probe, ensure_ascii=False) + "\n")
        out.flush()
        written += 1

        for i, chunk in enumerate(chunks[1:], 2):
            record = generate_pair(chunk, args.author, provider_fn, model)
            if record is not None:
                out.write(json.dumps(record, ensure_ascii=False) + "\n")
                out.flush()
                written += 1
                consecutive_fails = 0
            else:
                consecutive_fails += 1
                if consecutive_fails >= 10:
                    logger.error("10 подряд неудач — останавливаюсь. Проверь ключ/квоту.")
                    sys.exit(1)

            if i % 25 == 0:
                elapsed = time.time() - t_start
                rate = i / max(elapsed, 1e-9) * 60
                logger.info("  %d/%d (%.1f req/min, %d успешных)",
                            i, len(chunks), rate, written)

            time.sleep(sleep_between)

    logger.info("=== Готово: %d новых пар записано в %s ===", written, output_path)


if __name__ == "__main__":
    main()
