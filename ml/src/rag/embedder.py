"""Создание эмбеддингов через multilingual-e5-base."""

from sentence_transformers import SentenceTransformer

from src.config import settings


class Embedder:
    def __init__(self):
        self.model = SentenceTransformer(settings.EMBEDDING_MODEL)

    def embed_query(self, text: str) -> list[float]:
        return self.model.encode(f"query: {text}").tolist()

    def embed_passage(self, text: str) -> list[float]:
        return self.model.encode(f"passage: {text}").tolist()
