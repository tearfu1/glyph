"""Perplexity на held-out сабсете авторского корпуса.

Воспроизводит split из scripts/train_lora.py (seed=42, test_size=0.1,
block_size=512) и считает PPL = exp(mean cross-entropy loss) для заданной
модели (base или с LoRA-адаптером).
"""

from __future__ import annotations

import logging
import math
from pathlib import Path
from typing import Iterable

import torch
from datasets import Dataset

logger = logging.getLogger(__name__)

BLOCK_SIZE = 512
SEED = 42
TEST_SIZE = 0.1


def _load_author_text(processed_dir: Path, author: str) -> str:
    author_dir = processed_dir / author
    if not author_dir.exists():
        raise FileNotFoundError(f"Нет обработанных текстов: {author_dir}")
    texts = [f.read_text(encoding="utf-8") for f in sorted(author_dir.glob("*.txt"))]
    return "\n\n".join(texts)


def build_eval_dataset(processed_dir: Path, author: str, tokenizer) -> Dataset:
    """Возвращает held-out split (10%) из авторского корпуса."""
    text = _load_author_text(processed_dir, author)
    token_ids = tokenizer(text, return_attention_mask=False)["input_ids"]

    blocks = [
        {"input_ids": token_ids[i : i + BLOCK_SIZE]}
        for i in range(0, len(token_ids) - BLOCK_SIZE, BLOCK_SIZE)
    ]
    ds = Dataset.from_list(blocks)
    split = ds.train_test_split(test_size=TEST_SIZE, seed=SEED)
    return split["test"]


def compute_perplexity(model, tokenizer, eval_ds: Iterable[dict], device: str = "cpu") -> float:
    """Считает perplexity на блоках фиксированной длины."""
    model.eval()
    total_loss = 0.0
    total_tokens = 0

    with torch.no_grad():
        for item in eval_ds:
            input_ids = torch.tensor([item["input_ids"]], device=device)
            labels = input_ids.clone()
            outputs = model(input_ids=input_ids, labels=labels)
            n_tokens = input_ids.shape[1] - 1  # сдвиг на 1 для авторегрессии
            total_loss += outputs.loss.item() * n_tokens
            total_tokens += n_tokens

    mean_loss = total_loss / max(total_tokens, 1)
    return math.exp(mean_loss)
