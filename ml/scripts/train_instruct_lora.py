"""Instruction-tuning LoRA на synthetic датасете (этап C).

Отличия от train_lora.py:
- Обучается на data/synthetic/{author}.jsonl вместо сырых текстов
- Формат: "Вопрос: <q>\\nОтвет: <a>" с маскированием префикса в labels
  (loss считается только на токенах ответа)
- Гиперпараметры — копия E3_high_rank (r=16, alpha=32, epochs=5, lr=2e-4),
  подтверждённого как лучший в этапе 5
- Сохраняет в models/adapters/{author}/E5_instruct/

Использование (Colab T4):
    python -m scripts.train_instruct_lora --author dostoevsky
    python -m scripts.train_instruct_lora  # все 3 автора по очереди
"""

from __future__ import annotations

import argparse
import json
import logging
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, TaskType, get_peft_model
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
)

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

BASE_DIR = Path(__file__).resolve().parent.parent
SYNTHETIC_DIR = BASE_DIR / "data" / "synthetic"
ADAPTERS_DIR = BASE_DIR / "models" / "adapters"
RESULTS_FILE = BASE_DIR / "models" / "experiment_results_instruct.jsonl"

MODEL_NAME = "ai-forever/rugpt3small_based_on_gpt2"
MAX_LEN = 384
EXPERIMENT_NAME = "E5_instruct"
LORA_PARAMS = {"r": 16, "lora_alpha": 32, "lr": 2e-4, "epochs": 5}

AUTHORS = ["dostoevsky", "chekhov", "bulgakov"]


def load_synthetic(author: str) -> list[dict]:
    path = SYNTHETIC_DIR / f"{author}.jsonl"
    if not path.exists():
        raise FileNotFoundError(f"Нет synthetic датасета: {path}. Сначала запусти generate_synthetic_dataset.py")
    with open(path, "r", encoding="utf-8") as f:
        records = [json.loads(line) for line in f if line.strip()]
    logger.info("Автор %s: загружено %d пар", author, len(records))
    return records


def build_example(record: dict, tokenizer) -> dict | None:
    """Токенизирует пару с маскированием префикса в labels."""
    prompt = f"Вопрос: {record['question']}\nОтвет: "
    full_text = prompt + record["answer"] + tokenizer.eos_token

    full_ids = tokenizer(full_text, add_special_tokens=False)["input_ids"]
    prompt_ids = tokenizer(prompt, add_special_tokens=False)["input_ids"]
    prompt_len = len(prompt_ids)

    if len(full_ids) > MAX_LEN:
        full_ids = full_ids[:MAX_LEN]
    if prompt_len >= len(full_ids):
        return None  # слишком короткий ответ — пропускаем

    labels = list(full_ids)
    for i in range(prompt_len):
        labels[i] = -100

    return {"input_ids": full_ids, "labels": labels}


def build_dataset(records: list[dict], tokenizer) -> Dataset:
    examples = [build_example(r, tokenizer) for r in records]
    examples = [e for e in examples if e is not None]
    logger.info("Годных примеров после токенизации: %d", len(examples))
    return Dataset.from_list(examples)


def collate_fn(batch: list[dict], pad_token_id: int):
    """Pad input_ids/labels до max длины в батче. labels=-100 игнорируются в loss."""
    max_len = max(len(x["input_ids"]) for x in batch)
    n = len(batch)
    input_ids = torch.full((n, max_len), pad_token_id, dtype=torch.long)
    attention_mask = torch.zeros((n, max_len), dtype=torch.long)
    labels = torch.full((n, max_len), -100, dtype=torch.long)

    for i, item in enumerate(batch):
        L = len(item["input_ids"])
        input_ids[i, :L] = torch.tensor(item["input_ids"], dtype=torch.long)
        attention_mask[i, :L] = 1
        labels[i, :L] = torch.tensor(item["labels"], dtype=torch.long)

    return {"input_ids": input_ids, "attention_mask": attention_mask, "labels": labels}


def train_author(author: str, tokenizer, base_model) -> dict:
    logger.info("=== Instruct-tuning: %s (r=%d, alpha=%d, lr=%s, epochs=%d) ===",
                author, LORA_PARAMS["r"], LORA_PARAMS["lora_alpha"],
                LORA_PARAMS["lr"], LORA_PARAMS["epochs"])

    records = load_synthetic(author)
    dataset = build_dataset(records, tokenizer)
    split = dataset.train_test_split(test_size=0.1, seed=42)

    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=LORA_PARAMS["r"],
        lora_alpha=LORA_PARAMS["lora_alpha"],
        lora_dropout=0.05,
        target_modules=["c_attn", "c_proj"],
        bias="none",
    )
    model = get_peft_model(base_model, lora_config)
    model.print_trainable_parameters()

    output_dir = ADAPTERS_DIR / author / EXPERIMENT_NAME
    output_dir.mkdir(parents=True, exist_ok=True)

    training_args = TrainingArguments(
        output_dir=str(output_dir),
        num_train_epochs=LORA_PARAMS["epochs"],
        per_device_train_batch_size=8,
        per_device_eval_batch_size=8,
        learning_rate=LORA_PARAMS["lr"],
        warmup_steps=50,
        logging_steps=25,
        eval_strategy="epoch",
        save_strategy="epoch",
        save_total_limit=1,
        load_best_model_at_end=True,
        metric_for_best_model="eval_loss",
        fp16=torch.cuda.is_available(),
        report_to="none",
        dataloader_num_workers=2,
    )

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=split["train"],
        eval_dataset=split["test"],
        data_collator=lambda b: collate_fn(b, tokenizer.pad_token_id),
    )

    result = trainer.train()
    eval_result = trainer.evaluate()

    model.save_pretrained(str(output_dir))
    logger.info("Адаптер сохранён: %s", output_dir)

    record = {
        "author": author,
        "experiment": EXPERIMENT_NAME,
        **LORA_PARAMS,
        "train_loss": result.training_loss,
        "eval_loss": eval_result["eval_loss"],
        "train_samples": len(split["train"]),
        "eval_samples": len(split["test"]),
    }
    logger.info("Результат: train_loss=%.4f, eval_loss=%.4f",
                record["train_loss"], record["eval_loss"])

    RESULTS_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(RESULTS_FILE, "a", encoding="utf-8") as f:
        f.write(json.dumps(record, ensure_ascii=False) + "\n")

    del model, trainer
    if torch.cuda.is_available():
        torch.cuda.empty_cache()

    return record


def main():
    parser = argparse.ArgumentParser(description="Instruction tuning LoRA")
    parser.add_argument("--author", choices=AUTHORS,
                        help="Обучить только этого автора (default: все)")
    args = parser.parse_args()

    authors = [args.author] if args.author else AUTHORS

    logger.info("Загрузка базовой модели %s...", MODEL_NAME)
    tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    results = []
    for author in authors:
        base_model = AutoModelForCausalLM.from_pretrained(MODEL_NAME)
        record = train_author(author, tokenizer, base_model)
        results.append(record)
        del base_model
        if torch.cuda.is_available():
            torch.cuda.empty_cache()

    logger.info("=== ИТОГО ===")
    for r in results:
        logger.info("  %-12s  train_loss=%.4f  eval_loss=%.4f",
                    r["author"], r["train_loss"], r["eval_loss"])


if __name__ == "__main__":
    main()
