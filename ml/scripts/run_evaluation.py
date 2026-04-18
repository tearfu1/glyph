"""Оркестратор этапа 9: perplexity + стилевые + semantic метрики.

Условия эксперимента (5 штук):
  - baseline: base rugpt3small, без контекста RAG
  - rag_only: base rugpt3small + top-5 контекстных чанков
  - rag_lora_r4:  LoRA E2_low_rank + RAG
  - rag_lora_r8:  LoRA E1_default  + RAG
  - rag_lora_r16: LoRA E3_high_rank + RAG

Сохраняет результаты в models/evaluation/:
  - perplexity.csv          (автор × модель → PPL)
  - generation_samples.jsonl (все сгенерированные ответы + источники)
  - style_metrics.csv        (автор × условие → TTR, avg_sent_len, hapax, POS)
  - semantic_similarity.csv  (автор × условие → средний cosine)

Запуск:
    # Полный прогон на локальном CPU (медленно, ~2-3 часа)
    docker compose run --rm ml-service python -m scripts.run_evaluation

    # Только подготовка retrieval-кэша (нужен работающий Qdrant, ~2 мин)
    docker compose run --rm ml-service python -m scripts.run_evaluation \\
        --prepare-retrieval-cache data/retrieval_cache.json

    # В Colab с GPU (быстро, ~15 мин) — читает кэш вместо Qdrant
    python -m scripts.run_evaluation --use-cache data/retrieval_cache.json
"""

from __future__ import annotations

import argparse
import csv
import json
import logging
import time
from pathlib import Path

import torch
from peft import PeftModel
from transformers import AutoModelForCausalLM, AutoTokenizer

from src.config import settings
from src.evaluation.perplexity import build_eval_dataset, compute_perplexity
from src.evaluation.questions import EVAL_QUESTIONS, author_questions
from src.evaluation.semantic_score import compute_semantic_similarity
from src.evaluation.style_metrics import POS_TAGS, aggregate_style_metrics

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

BASE_DIR = Path(__file__).resolve().parent.parent
PROCESSED_DIR = BASE_DIR / "data" / "processed"
ADAPTERS_DIR = BASE_DIR / "models" / "adapters"
RESULTS_DIR = BASE_DIR / "models" / "evaluation"

EXPERIMENTS = ["E1_default", "E2_low_rank", "E3_high_rank", "E4_long_train"]

CONDITIONS = [
    ("baseline",     None,              False),
    ("rag_only",     None,              True),
    ("rag_lora_r4",  "E2_low_rank",     True),
    ("rag_lora_r8",  "E1_default",      True),
    ("rag_lora_r16", "E3_high_rank",    True),
]

AUTHORS = list(EVAL_QUESTIONS.keys())


def _new_base_model(device: str):
    model = AutoModelForCausalLM.from_pretrained(settings.MODEL_NAME)
    model.to(device)
    model.eval()
    return model


def _attach_adapter(base, author: str, experiment: str, device: str):
    adapter_path = ADAPTERS_DIR / author / experiment
    if not adapter_path.exists():
        raise FileNotFoundError(f"Адаптер не найден: {adapter_path}")
    model = PeftModel.from_pretrained(base, str(adapter_path))
    model.to(device)
    model.eval()
    return model


def run_perplexity(authors: list[str], device: str) -> list[dict]:
    """PPL на held-out 10% для base и всех 4 адаптеров на каждого автора."""
    logger.info("=== Perplexity ===")
    tokenizer = AutoTokenizer.from_pretrained(settings.MODEL_NAME)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    records: list[dict] = []
    for author in authors:
        logger.info("Автор: %s", author)
        eval_ds = build_eval_dataset(PROCESSED_DIR, author, tokenizer)
        logger.info("Eval блоков: %d", len(eval_ds))

        base = _new_base_model(device)
        ppl = compute_perplexity(base, tokenizer, eval_ds, device)
        logger.info("  baseline PPL = %.2f", ppl)
        records.append({"author": author, "model": "baseline", "perplexity": ppl})
        del base
        if device == "cuda":
            torch.cuda.empty_cache()

        for exp in EXPERIMENTS:
            if not (ADAPTERS_DIR / author / exp).exists():
                logger.warning("  пропуск %s (адаптер не найден)", exp)
                continue
            base = _new_base_model(device)
            model = _attach_adapter(base, author, exp, device)
            ppl = compute_perplexity(model, tokenizer, eval_ds, device)
            logger.info("  %s PPL = %.2f", exp, ppl)
            records.append({"author": author, "model": exp, "perplexity": ppl})
            del model, base
            if device == "cuda":
                torch.cuda.empty_cache()

    return records


def _build_prompt(question: str, context_chunks: list[str] | None) -> str:
    if context_chunks:
        ctx = "\n\n".join(context_chunks)
        return (
            f"Контекст из произведений автора:\n{ctx}\n\n"
            f"Вопрос читателя: {question}\n\n"
            f"Ответ в стиле автора:"
        )
    return f"Вопрос читателя: {question}\n\nОтвет в стиле автора:"


def _generate(model, tokenizer, prompt: str, device: str) -> str:
    inputs = tokenizer(prompt, return_tensors="pt", truncation=True, max_length=1024).to(device)
    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=256,
            temperature=0.8,
            top_p=0.9,
            repetition_penalty=1.2,
            do_sample=True,
            pad_token_id=tokenizer.eos_token_id,
        )
    answer_tokens = outputs[0][inputs["input_ids"].shape[1]:]
    return tokenizer.decode(answer_tokens, skip_special_tokens=True).strip()


def prepare_retrieval_cache(authors: list[str], num_questions: int, out_path: Path) -> dict:
    """Один проход через Qdrant: сохраняет retrieval для всех вопросов в JSON.

    Формат: {author: {question: [{text, score, book, chunk_id}, ...]}}
    """
    from src.rag.embedder import Embedder
    from src.rag.retriever import Retriever

    logger.info("=== Подготовка retrieval-кэша ===")
    embedder = Embedder()
    retriever = Retriever(embedder)

    cache: dict[str, dict[str, list[dict]]] = {}
    for author in authors:
        cache[author] = {}
        questions = author_questions(author)[:num_questions]
        logger.info("Автор %s: %d вопросов", author, len(questions))
        for q in questions:
            try:
                cache[author][q] = retriever.retrieve(q, author, top_k=settings.TOP_K)
            except Exception as e:
                logger.warning("Retrieval упал для '%s': %s", q[:40], e)
                cache[author][q] = []

    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(cache, f, ensure_ascii=False, indent=2)
    logger.info("Кэш сохранён: %s", out_path)
    return cache


def _load_retrieval_cache(path: Path) -> dict[str, dict[str, list[dict]]]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def run_generation(
    authors: list[str],
    num_questions: int,
    device: str,
    retrieval_cache: dict | None = None,
):
    """Генерирует ответы для всех условий и считает метрики.

    retrieval_cache: если передан — использует готовые чанки вместо Qdrant.

    Возвращает (samples, style_records, semantic_records).
    """
    logger.info("=== Генерация + метрики ===")
    from src.rag.embedder import Embedder

    tokenizer = AutoTokenizer.from_pretrained(settings.MODEL_NAME)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    embedder = Embedder()
    retriever = None
    if retrieval_cache is None:
        from src.rag.retriever import Retriever
        retriever = Retriever(embedder)

    samples: list[dict] = []
    style_records: list[dict] = []
    semantic_records: list[dict] = []

    for author in authors:
        questions = author_questions(author)[:num_questions]
        logger.info("Автор %s: %d вопросов", author, len(questions))

        # Retrieval один раз на вопрос (контекст одинаков для всех RAG-условий)
        retrieved: dict[str, list[dict]] = {}
        for q in questions:
            if retrieval_cache is not None:
                retrieved[q] = retrieval_cache.get(author, {}).get(q, [])
                if not retrieved[q]:
                    logger.warning("В кэше нет retrieval для '%s'", q[:40])
                continue
            try:
                retrieved[q] = retriever.retrieve(q, author, top_k=settings.TOP_K)
            except Exception as e:
                logger.warning("Retrieval упал для '%s': %s", q[:40], e)
                retrieved[q] = []

        for cond_name, experiment, use_rag in CONDITIONS:
            logger.info("  Условие: %s", cond_name)
            # Подбор модели
            base = _new_base_model(device)
            if experiment is not None:
                adapter_dir = ADAPTERS_DIR / author / experiment
                if not adapter_dir.exists():
                    logger.warning("    пропуск (нет адаптера %s)", adapter_dir)
                    del base
                    continue
                model = _attach_adapter(base, author, experiment, device)
            else:
                model = base

            answers: list[str] = []
            semantic_scores: list[float] = []
            t0 = time.time()
            for q in questions:
                chunks = [c["text"] for c in retrieved[q]] if use_rag else None
                prompt = _build_prompt(q, chunks)
                answer = _generate(model, tokenizer, prompt, device)
                answers.append(answer)

                if use_rag and chunks:
                    semantic_scores.append(
                        compute_semantic_similarity(answer, chunks, embedder)
                    )

                samples.append({
                    "author": author,
                    "condition": cond_name,
                    "question": q,
                    "answer": answer,
                    "sources": retrieved[q] if use_rag else [],
                })
            dt = time.time() - t0
            logger.info("    %d ответов за %.1f сек (%.1f сек/ответ)",
                        len(answers), dt, dt / max(len(answers), 1))

            # Стилевые метрики (усреднённые по ответам)
            style = aggregate_style_metrics(answers)
            row = {
                "author": author,
                "condition": cond_name,
                "ttr": style["ttr"],
                "avg_sentence_length": style["avg_sentence_length"],
                "hapax_ratio": style["hapax_ratio"],
            }
            row.update({f"pos_{tag}": style["pos"][tag] for tag in POS_TAGS})
            style_records.append(row)

            if use_rag and semantic_scores:
                semantic_records.append({
                    "author": author,
                    "condition": cond_name,
                    "semantic_similarity": sum(semantic_scores) / len(semantic_scores),
                    "n": len(semantic_scores),
                })

            del model
            if experiment is not None:
                del base
            if device == "cuda":
                torch.cuda.empty_cache()

    # Эталон: метрики самого авторского корпуса (для сравнения с генерацией)
    logger.info("=== Стилевые метрики оригинала ===")
    for author in authors:
        author_dir = PROCESSED_DIR / author
        texts = [f.read_text(encoding="utf-8")[:50_000] for f in sorted(author_dir.glob("*.txt"))]
        style = aggregate_style_metrics(texts)
        row = {
            "author": author,
            "condition": "original",
            "ttr": style["ttr"],
            "avg_sentence_length": style["avg_sentence_length"],
            "hapax_ratio": style["hapax_ratio"],
        }
        row.update({f"pos_{tag}": style["pos"][tag] for tag in POS_TAGS})
        style_records.append(row)

    return samples, style_records, semantic_records


def write_csv(path: Path, rows: list[dict]):
    if not rows:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = list(rows[0].keys())
    with open(path, "w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    logger.info("Сохранено: %s (%d строк)", path, len(rows))


def write_jsonl(path: Path, rows: list[dict]):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    logger.info("Сохранено: %s (%d строк)", path, len(rows))


def main():
    parser = argparse.ArgumentParser(description="Evaluation pipeline (этап 9)")
    parser.add_argument("--authors", nargs="+", choices=AUTHORS, default=AUTHORS)
    parser.add_argument("--num-questions", type=int, default=10,
                        help="Сколько вопросов на автора использовать (default: 10)")
    parser.add_argument("--skip-perplexity", action="store_true")
    parser.add_argument("--skip-generation", action="store_true")
    parser.add_argument("--device", default=None,
                        help="cpu/cuda (default: авто)")
    parser.add_argument("--prepare-retrieval-cache", metavar="PATH",
                        help="Только retrieval через Qdrant, сохранить в JSON и выйти")
    parser.add_argument("--use-cache", metavar="PATH",
                        help="Читать retrieval из JSON-кэша вместо Qdrant (для Colab без Qdrant)")
    args = parser.parse_args()

    device = args.device or ("cuda" if torch.cuda.is_available() else "cpu")
    logger.info("Device: %s", device)
    logger.info("Authors: %s", args.authors)

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    if args.prepare_retrieval_cache:
        prepare_retrieval_cache(
            args.authors, args.num_questions, Path(args.prepare_retrieval_cache)
        )
        logger.info("=== Retrieval-кэш готов ===")
        return

    retrieval_cache = None
    if args.use_cache:
        logger.info("Читаю retrieval-кэш: %s", args.use_cache)
        retrieval_cache = _load_retrieval_cache(Path(args.use_cache))

    if not args.skip_perplexity:
        ppl_records = run_perplexity(args.authors, device)
        write_csv(RESULTS_DIR / "perplexity.csv", ppl_records)

    if not args.skip_generation:
        samples, style_records, semantic_records = run_generation(
            args.authors, args.num_questions, device, retrieval_cache=retrieval_cache
        )
        write_jsonl(RESULTS_DIR / "generation_samples.jsonl", samples)
        write_csv(RESULTS_DIR / "style_metrics.csv", style_records)
        write_csv(RESULTS_DIR / "semantic_similarity.csv", semantic_records)

    logger.info("=== Evaluation завершена ===")


if __name__ == "__main__":
    main()
