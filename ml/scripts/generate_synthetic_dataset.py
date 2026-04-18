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
CHUNKS_DIR = BASE_DIR / "data" / "chunks"
OUTPUT_DIR = BASE_DIR / "data" / "synthetic"

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

def call_groq(messages: list[dict], model: str = "llama-3.3-70b-versatile") -> str:
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
    "groq": "llama-3.3-70b-versatile",
    "gemini": "gemini-2.0-flash",
    "anthropic": "claude-haiku-4-5",
}


# ---------- Core logic ----------

def load_chunks(author: str) -> list[dict]:
    """Загружает все чанки автора из JSONL файлов."""
    author_dir = CHUNKS_DIR / author
    if not author_dir.exists():
        raise FileNotFoundError(f"Нет чанков автора: {author_dir}")

    chunks: list[dict] = []
    for jsonl_path in sorted(author_dir.glob("*.jsonl")):
        with open(jsonl_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line:
                    chunks.append(json.loads(line))
    logger.info("Автор %s: загружено %d чанков", author, len(chunks))
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
) -> dict | None:
    """Один вызов LLM → распарсенная пара (question, answer). None при ошибке."""
    author_ru = AUTHOR_NAMES_RU.get(author, author)
    user_prompt = USER_PROMPT_TEMPLATE.format(
        author_ru=author_ru,
        chunk_text=chunk["text"],
    )
    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": user_prompt},
    ]

    try:
        raw = provider_fn(messages, model)
    except Exception as e:
        logger.warning("LLM call failed (chunk %s/%s): %s",
                       chunk.get("book"), chunk.get("chunk_id"), e)
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
    parser.add_argument("--rpm", type=int, default=30,
                        help="rate limit — запросов в минуту (default 30)")
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

    sleep_between = max(60.0 / args.rpm, 0.0)
    written = 0
    t_start = time.time()

    with open(output_path, "a", encoding="utf-8") as out:
        for i, chunk in enumerate(chunks, 1):
            record = generate_pair(chunk, args.author, provider_fn, model)
            if record is not None:
                out.write(json.dumps(record, ensure_ascii=False) + "\n")
                out.flush()
                written += 1

            if i % 25 == 0:
                elapsed = time.time() - t_start
                rate = i / max(elapsed, 1e-9) * 60
                logger.info("  %d/%d (%.1f req/min, %d успешных)",
                            i, len(chunks), rate, written)

            time.sleep(sleep_between)

    logger.info("=== Готово: %d новых пар записано в %s ===", written, output_path)


if __name__ == "__main__":
    main()
