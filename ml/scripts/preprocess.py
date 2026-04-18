"""Очистка сырых текстов и разбиение на чанки (JSONL)."""

import json
import logging
import re
from pathlib import Path

from tokenizers import Tokenizer

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

DATA_DIR = Path(__file__).resolve().parent.parent / "data"
RAW_DIR = DATA_DIR / "raw"
PROCESSED_DIR = DATA_DIR / "processed"
CHUNKS_DIR = DATA_DIR / "chunks"

CHUNK_SIZE = 256  # токенов
CHUNK_OVERLAP = 64


def clean_text(text: str) -> str:
    """Очистка текста от HTML-артефактов и мусора."""
    # HTML-сущности
    text = re.sub(r"&nbsp;", " ", text)
    text = re.sub(r"&#\d+;", "", text)
    text = re.sub(r"&\w+;", "", text)
    # Служебные пометки lib.ru
    text = re.sub(r"\[(\d+)\]", "", text)  # сноски [1], [2]
    # Нормализация пробелов
    text = re.sub(r"[ \t]+", " ", text)
    # Нормализация переносов строк (оставляем двойные как разделители абзацев)
    text = re.sub(r"\n{3,}", "\n\n", text)
    # Убираем пробелы в начале/конце строк
    text = "\n".join(line.strip() for line in text.split("\n"))
    return text.strip()


def chunk_text(
    text: str, tokenizer: Tokenizer, chunk_size: int = CHUNK_SIZE, overlap: int = CHUNK_OVERLAP
) -> list[str]:
    """Разбиение текста на чанки по токенам с перекрытием."""
    encoding = tokenizer.encode(text)
    token_ids = encoding.ids
    chunks = []
    start = 0
    while start < len(token_ids):
        end = min(start + chunk_size, len(token_ids))
        chunk_ids = token_ids[start:end]
        chunk_text = tokenizer.decode(chunk_ids)
        chunks.append(chunk_text.strip())
        if end >= len(token_ids):
            break
        start += chunk_size - overlap
    return chunks


def main():
    # Загружаем токенизатор rugpt3small из HuggingFace Hub
    logger.info("Загрузка токенизатора gpt2 (совместим с rugpt3small)...")
    tokenizer = Tokenizer.from_pretrained("gpt2")

    total_stats = {}

    for author_dir in sorted(RAW_DIR.iterdir()):
        if not author_dir.is_dir():
            continue
        author = author_dir.name
        proc_dir = PROCESSED_DIR / author
        proc_dir.mkdir(parents=True, exist_ok=True)
        chunk_dir = CHUNKS_DIR / author
        chunk_dir.mkdir(parents=True, exist_ok=True)

        author_chunks = 0

        for raw_file in sorted(author_dir.glob("*.txt")):
            book = raw_file.stem

            # 1. Очистка
            raw_text = raw_file.read_text(encoding="utf-8")
            cleaned = clean_text(raw_text)
            proc_path = proc_dir / f"{book}.txt"
            proc_path.write_text(cleaned, encoding="utf-8")

            # 2. Чанкинг
            chunks = chunk_text(cleaned, tokenizer)
            chunk_path = chunk_dir / f"{book}_chunks.jsonl"
            with open(chunk_path, "w", encoding="utf-8") as f:
                for i, chunk in enumerate(chunks):
                    record = {
                        "text": chunk,
                        "book": book,
                        "author": author,
                        "chunk_id": i,
                    }
                    f.write(json.dumps(record, ensure_ascii=False) + "\n")

            author_chunks += len(chunks)
            logger.info(
                "%s / %s: %d символов → %d чанков",
                author, book, len(cleaned), len(chunks),
            )

        total_stats[author] = author_chunks
        logger.info("Итого %s: %d чанков", author, author_chunks)

    logger.info("=== Общая статистика ===")
    for author, count in total_stats.items():
        logger.info("  %s: %d чанков", author, count)


if __name__ == "__main__":
    main()
