"""Индексация чанков в Qdrant: эмбеддинги через multilingual-e5-base."""

import json
import logging
import uuid
from pathlib import Path

from qdrant_client import QdrantClient
from qdrant_client.models import Distance, PointStruct, VectorParams
from sentence_transformers import SentenceTransformer
from tqdm import tqdm

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

CHUNKS_DIR = Path(__file__).resolve().parent.parent / "data" / "chunks"
COLLECTION_NAME = "glyph_chunks"
QDRANT_HOST = "localhost"
QDRANT_PORT = 6333
EMBEDDING_MODEL = "intfloat/multilingual-e5-base"
VECTOR_SIZE = 768
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
    # 1. Загрузка модели
    logger.info("Загрузка модели %s...", EMBEDDING_MODEL)
    model = SentenceTransformer(EMBEDDING_MODEL)

    # 2. Загрузка чанков
    chunks = load_all_chunks()
    logger.info("Загружено %d чанков", len(chunks))

    # 3. Подключение к Qdrant и создание коллекции
    client = QdrantClient(host=QDRANT_HOST, port=QDRANT_PORT)
    client.recreate_collection(
        collection_name=COLLECTION_NAME,
        vectors_config=VectorParams(size=VECTOR_SIZE, distance=Distance.COSINE),
    )
    logger.info("Коллекция '%s' создана", COLLECTION_NAME)

    # 4. Создание эмбеддингов и загрузка батчами
    for i in tqdm(range(0, len(chunks), BATCH_SIZE), desc="Индексация"):
        batch = chunks[i : i + BATCH_SIZE]
        # e5 требует префикс "passage: " для документов
        texts = [f"passage: {c['text']}" for c in batch]
        embeddings = model.encode(texts, show_progress_bar=False)

        points = [
            PointStruct(
                id=str(uuid.uuid4()),
                vector=emb.tolist(),
                payload={
                    "author": c["author"],
                    "book": c["book"],
                    "chunk_id": c["chunk_id"],
                    "text": c["text"],
                },
            )
            for c, emb in zip(batch, embeddings)
        ]
        client.upsert(collection_name=COLLECTION_NAME, points=points)

    # 5. Проверка
    info = client.get_collection(COLLECTION_NAME)
    logger.info("Индексация завершена. Точек в коллекции: %d", info.points_count)


if __name__ == "__main__":
    main()
