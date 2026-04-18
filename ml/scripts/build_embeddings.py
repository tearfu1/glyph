"""Создание эмбеддингов чанков (запускать в Google Colab).

Читает JSONL чанки, создаёт эмбеддинги через multilingual-e5-base,
сохраняет в .npy + метаданные в .jsonl.

Использование в Colab:
    1. Загрузить папку data/chunks/ в Colab (или смонтировать Google Drive)
    2. pip install sentence-transformers
    3. python build_embeddings.py
    4. Скачать data/embeddings/ обратно на локальную машину
"""

import json
import logging
from pathlib import Path

import numpy as np
from sentence_transformers import SentenceTransformer
from tqdm import tqdm

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

DATA_DIR = Path(__file__).resolve().parent.parent / "data"
CHUNKS_DIR = DATA_DIR / "chunks"
EMBEDDINGS_DIR = DATA_DIR / "embeddings"
EMBEDDING_MODEL = "intfloat/multilingual-e5-base"
BATCH_SIZE = 64


def load_all_chunks() -> list[dict]:
    """Загрузка всех чанков из JSONL файлов."""
    chunks = []
    for author_dir in sorted(CHUNKS_DIR.iterdir()):
        if not author_dir.is_dir():
            continue
        for jsonl_file in sorted(author_dir.glob("*_chunks.jsonl")):
            with open(jsonl_file, encoding="utf-8") as f:
                for line in f:
                    chunks.append(json.loads(line))
    return chunks


def main():
    EMBEDDINGS_DIR.mkdir(parents=True, exist_ok=True)

    # 1. Загрузка модели
    logger.info("Загрузка модели %s...", EMBEDDING_MODEL)
    model = SentenceTransformer(EMBEDDING_MODEL)

    # 2. Загрузка чанков
    chunks = load_all_chunks()
    logger.info("Загружено %d чанков", len(chunks))

    # 3. Создание эмбеддингов батчами
    # e5 требует префикс "passage: " для документов
    texts = [f"passage: {c['text']}" for c in chunks]
    logger.info("Создание эмбеддингов...")
    embeddings = model.encode(texts, batch_size=BATCH_SIZE, show_progress_bar=True)

    # 4. Сохранение
    np.save(EMBEDDINGS_DIR / "embeddings.npy", embeddings)
    logger.info("Эмбеддинги сохранены: %s, shape=%s", "embeddings.npy", embeddings.shape)

    with open(EMBEDDINGS_DIR / "metadata.jsonl", "w", encoding="utf-8") as f:
        for c in chunks:
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    logger.info("Метаданные сохранены: metadata.jsonl (%d записей)", len(chunks))


if __name__ == "__main__":
    main()
