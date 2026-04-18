"""LoRA fine-tuning rugpt3small на текстах авторов (запускать в Google Colab).

Для каждого автора:
1. Конкатенирует все очищенные тексты
2. Токенизирует и нарезает на блоки по 512 токенов
3. Обучает LoRA-адаптер
4. Сохраняет адаптер в models/adapters/{author}/

Использование в Colab:
    1. Загрузить data/processed/ в Google Drive
    2. pip install transformers peft datasets accelerate
    3. python train_lora.py --author dostoevsky
    4. Скачать models/adapters/ обратно
"""

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
    DataCollatorForLanguageModeling,
    Trainer,
    TrainingArguments,
)

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

BASE_DIR = Path(__file__).resolve().parent.parent
PROCESSED_DIR = BASE_DIR / "data" / "processed"
ADAPTERS_DIR = BASE_DIR / "models" / "adapters"
RESULTS_FILE = BASE_DIR / "models" / "experiment_results.jsonl"

MODEL_NAME = "ai-forever/rugpt3small_based_on_gpt2"
BLOCK_SIZE = 512

EXPERIMENTS = {
    "E1_default":    {"r": 8,  "lora_alpha": 16, "lr": 2e-4, "epochs": 5},
    "E2_low_rank":   {"r": 4,  "lora_alpha": 8,  "lr": 2e-4, "epochs": 5},
    "E3_high_rank":  {"r": 16, "lora_alpha": 32, "lr": 2e-4, "epochs": 5},
    "E4_long_train": {"r": 8,  "lora_alpha": 16, "lr": 1e-4, "epochs": 10},
}

AUTHORS = ["dostoevsky", "chekhov", "bulgakov"]


def load_author_texts(author: str) -> str:
    """Конкатенация всех текстов автора."""
    author_dir = PROCESSED_DIR / author
    if not author_dir.exists():
        raise FileNotFoundError(f"Нет данных для автора: {author_dir}")

    texts = []
    for f in sorted(author_dir.glob("*.txt")):
        texts.append(f.read_text(encoding="utf-8"))
    full_text = "\n\n".join(texts)
    logger.info("Автор %s: %d файлов, %d символов", author, len(texts), len(full_text))
    return full_text


def prepare_dataset(text: str, tokenizer) -> Dataset:
    """Токенизация текста и нарезка на блоки фиксированной длины."""
    tokenized = tokenizer(text, return_attention_mask=False)
    input_ids = tokenized["input_ids"]

    blocks = []
    for i in range(0, len(input_ids) - BLOCK_SIZE, BLOCK_SIZE):
        block = input_ids[i : i + BLOCK_SIZE]
        blocks.append({"input_ids": block, "labels": block})

    logger.info("Подготовлено %d блоков по %d токенов", len(blocks), BLOCK_SIZE)
    return Dataset.from_list(blocks)


def train_single(author: str, experiment_name: str, params: dict, tokenizer, base_model):
    """Обучение одного LoRA-адаптера."""
    logger.info("=== %s / %s (r=%d, alpha=%d, lr=%s, epochs=%d) ===",
                author, experiment_name, params["r"], params["lora_alpha"],
                params["lr"], params["epochs"])

    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=params["r"],
        lora_alpha=params["lora_alpha"],
        lora_dropout=0.05,
        target_modules=["c_attn", "c_proj"],
        bias="none",
    )

    model = get_peft_model(base_model, lora_config)
    model.print_trainable_parameters()

    text = load_author_texts(author)
    dataset = prepare_dataset(text, tokenizer)
    split = dataset.train_test_split(test_size=0.1, seed=42)

    output_dir = ADAPTERS_DIR / author / experiment_name
    output_dir.mkdir(parents=True, exist_ok=True)

    training_args = TrainingArguments(
        output_dir=str(output_dir),
        num_train_epochs=params["epochs"],
        per_device_train_batch_size=8,
        per_device_eval_batch_size=8,
        learning_rate=params["lr"],
        warmup_steps=100,
        logging_steps=50,
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
        data_collator=DataCollatorForLanguageModeling(tokenizer, mlm=False),
    )

    result = trainer.train()
    eval_result = trainer.evaluate()

    # Сохранение адаптера
    model.save_pretrained(str(output_dir))
    logger.info("Адаптер сохранён: %s", output_dir)

    record = {
        "author": author,
        "experiment": experiment_name,
        **params,
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
    parser = argparse.ArgumentParser(description="LoRA fine-tuning для Glyph")
    parser.add_argument("--author", choices=AUTHORS, help="Обучить только этого автора")
    parser.add_argument("--experiment", choices=list(EXPERIMENTS.keys()),
                        help="Запустить только этот эксперимент")
    parser.add_argument("--best-only", action="store_true",
                        help="Обучить только E1_default (для быстрого прототипа)")
    args = parser.parse_args()

    authors = [args.author] if args.author else AUTHORS
    if args.best_only:
        experiments = {"E1_default": EXPERIMENTS["E1_default"]}
    elif args.experiment:
        experiments = {args.experiment: EXPERIMENTS[args.experiment]}
    else:
        experiments = EXPERIMENTS

    logger.info("Загрузка базовой модели %s...", MODEL_NAME)
    tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    results = []
    for author in authors:
        for exp_name, params in experiments.items():
            base_model = AutoModelForCausalLM.from_pretrained(MODEL_NAME)
            record = train_single(author, exp_name, params, tokenizer, base_model)
            results.append(record)
            del base_model
            if torch.cuda.is_available():
                torch.cuda.empty_cache()

    logger.info("=== ИТОГОВАЯ ТАБЛИЦА ===")
    logger.info("%-12s %-15s %5s %5s %8s %8s", "Author", "Experiment", "r", "alpha",
                "TrainLoss", "EvalLoss")
    for r in results:
        logger.info("%-12s %-15s %5d %5d %8.4f %8.4f",
                    r["author"], r["experiment"], r["r"], r["lora_alpha"],
                    r["train_loss"], r["eval_loss"])


if __name__ == "__main__":
    main()
