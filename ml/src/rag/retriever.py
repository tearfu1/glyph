"""Поиск релевантных чанков в Qdrant."""

from qdrant_client import QdrantClient
from qdrant_client.models import FieldCondition, Filter, MatchValue

from src.config import settings
from src.rag.embedder import Embedder


class Retriever:
    def __init__(self, embedder: Embedder):
        self.embedder = embedder
        self.client = QdrantClient(host=settings.QDRANT_HOST, port=settings.QDRANT_PORT)

    def retrieve(self, query: str, author: str, top_k: int | None = None) -> list[dict]:
        top_k = top_k or settings.TOP_K
        vector = self.embedder.embed_query(query)

        results = self.client.search(
            collection_name="glyph_chunks",
            query_vector=vector,
            query_filter=Filter(must=[
                FieldCondition(key="author", match=MatchValue(value=author))
            ]),
            limit=top_k,
        )

        return [
            {
                "text": r.payload["text"],
                "score": r.score,
                "book": r.payload["book"],
                "chunk_id": r.payload["chunk_id"],
            }
            for r in results
        ]
