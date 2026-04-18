"""Semantic similarity между сгенерированным ответом и retrieved chunks."""

from __future__ import annotations

import numpy as np

from src.rag.embedder import Embedder


def _cosine(a: list[float], b: list[float]) -> float:
    va = np.asarray(a, dtype=np.float32)
    vb = np.asarray(b, dtype=np.float32)
    denom = float(np.linalg.norm(va) * np.linalg.norm(vb))
    if denom == 0.0:
        return 0.0
    return float(np.dot(va, vb) / denom)


def compute_semantic_similarity(
    answer: str,
    chunks: list[str],
    embedder: Embedder,
) -> float:
    """Средний cosine между embedding ответа и embedding-ами контекстных чанков."""
    if not answer.strip() or not chunks:
        return 0.0
    answer_emb = embedder.embed_query(answer)
    scores = [_cosine(answer_emb, embedder.embed_passage(c)) for c in chunks]
    return float(np.mean(scores))
