"""Загрузка эмбеддингов в Qdrant (запускать локально).

Читает .npy эмбеддинги и метаданные, загружает в Qdrant.
Не требует torch/sentence-transformers — только qdrant-client и numpy.

Использование:
    pip install qdrant-client numpy tqdm
    python upload_index.py
"""

import json
import logging
import uuid
from pathlib import Path

import numpy as np
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, PointStruct, VectorParams
from tqdm import tqdm

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

DATA_DIR = Path(__file__).resolve().parent.parent / "data"
EMBEDDINGS_DIR = DATA_DIR / "embeddings"
COLLECTION_NAME = "glyph_chunks"
QDRANT_HOST = "localhost"
QDRANT_PORT = 6333
VECTOR_SIZE = 768
BATCH_SIZE = 100


def main():
    # 1. Загрузка данных
    embeddings = np.load(EMBEDDINGS_DIR / "embeddings.npy")
    logger.info("Эмбеддинги загружены: shape=%s", embeddings.shape)

    metadata = []
    with open(EMBEDDINGS_DIR / "metadata.jsonl", encoding="utf-8") as f:
        for line in f:
            metadata.append(json.loads(line))
    logger.info("Метаданные загружены: %d записей", len(metadata))

    assert len(embeddings) == len(metadata), "Количество эмбеддингов и метаданных не совпадает"

    # 2. Подключение к Qdrant
    client = QdrantClient(host=QDRANT_HOST, port=QDRANT_PORT)
    client.recreate_collection(
        collection_name=COLLECTION_NAME,
        vectors_config=VectorParams(size=VECTOR_SIZE, distance=Distance.COSINE),
    )
    logger.info("Коллекция '%s' создана", COLLECTION_NAME)

    # 3. Загрузка батчами
    for i in tqdm(range(0, len(embeddings), BATCH_SIZE), desc="Загрузка в Qdrant"):
        batch_emb = embeddings[i : i + BATCH_SIZE]
        batch_meta = metadata[i : i + BATCH_SIZE]

        points = [
            PointStruct(
                id=str(uuid.uuid4()),
                vector=emb.tolist(),
                payload={
                    "author": m["author"],
                    "book": m["book"],
                    "chunk_id": m["chunk_id"],
                    "text": m["text"],
                },
            )
            for emb, m in zip(batch_emb, batch_meta)
        ]
        client.upsert(collection_name=COLLECTION_NAME, points=points)

    # 4. Проверка
    info = client.get_collection(COLLECTION_NAME)
    logger.info("Загрузка завершена. Точек в коллекции: %d", info.points_count)


if __name__ == "__main__":
    main()
