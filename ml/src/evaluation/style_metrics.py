"""Стилевые метрики текста для сравнения автор vs генерация.

Использует razdel для сегментации и pymorphy3 для POS-разметки.
"""

from __future__ import annotations

from collections import Counter
from functools import lru_cache
from typing import Iterable

import pymorphy3
from razdel import sentenize, tokenize

POS_TAGS = ("NOUN", "VERB", "ADJF", "ADVB", "PREP", "CONJ", "PRCL", "INFN", "PRTF", "GRND")


@lru_cache(maxsize=1)
def _morph() -> pymorphy3.MorphAnalyzer:
    return pymorphy3.MorphAnalyzer()


def _word_tokens(text: str) -> list[str]:
    return [t.text.lower() for t in tokenize(text) if any(ch.isalpha() for ch in t.text)]


def _sentences(text: str) -> list[str]:
    return [s.text for s in sentenize(text)]


def type_token_ratio(text: str) -> float:
    tokens = _word_tokens(text)
    if not tokens:
        return 0.0
    return len(set(tokens)) / len(tokens)


def avg_sentence_length(text: str) -> float:
    sents = _sentences(text)
    if not sents:
        return 0.0
    word_counts = [len(_word_tokens(s)) for s in sents]
    return sum(word_counts) / len(word_counts)


def hapax_ratio(text: str) -> float:
    tokens = _word_tokens(text)
    if not tokens:
        return 0.0
    counts = Counter(tokens)
    hapax = sum(1 for _, c in counts.items() if c == 1)
    return hapax / len(tokens)


def pos_distribution(text: str) -> dict[str, float]:
    tokens = _word_tokens(text)
    if not tokens:
        return {tag: 0.0 for tag in POS_TAGS}

    morph = _morph()
    pos_counts: Counter[str] = Counter()
    for tok in tokens:
        parse = morph.parse(tok)
        if not parse:
            continue
        tag = parse[0].tag.POS
        if tag:
            pos_counts[tag] += 1

    total = sum(pos_counts.values()) or 1
    return {tag: pos_counts.get(tag, 0) / total for tag in POS_TAGS}


def compute_style_metrics(text: str) -> dict:
    """Единый entry-point — вызывается из orchestrator."""
    metrics = {
        "ttr": type_token_ratio(text),
        "avg_sentence_length": avg_sentence_length(text),
        "hapax_ratio": hapax_ratio(text),
    }
    metrics["pos"] = pos_distribution(text)
    return metrics


def aggregate_style_metrics(texts: Iterable[str]) -> dict:
    """Усреднение метрик по коллекции текстов (например, 20 сгенерированных ответов)."""
    collected: list[dict] = [compute_style_metrics(t) for t in texts if t.strip()]
    if not collected:
        return {"ttr": 0.0, "avg_sentence_length": 0.0, "hapax_ratio": 0.0,
                "pos": {tag: 0.0 for tag in POS_TAGS}}

    n = len(collected)
    mean = {
        "ttr": sum(m["ttr"] for m in collected) / n,
        "avg_sentence_length": sum(m["avg_sentence_length"] for m in collected) / n,
        "hapax_ratio": sum(m["hapax_ratio"] for m in collected) / n,
        "pos": {
            tag: sum(m["pos"].get(tag, 0.0) for m in collected) / n
            for tag in POS_TAGS
        },
    }
    return mean
