"""Загрузка rugpt3small + LoRA адаптеры."""

import logging
from pathlib import Path

from peft import PeftModel
from transformers import AutoModelForCausalLM, AutoTokenizer

from src.config import settings

logger = logging.getLogger(__name__)

ADAPTERS_DIR = Path("models/adapters")


class ModelManager:
    def __init__(self):
        logger.info("Загрузка базовой модели %s...", settings.MODEL_NAME)
        self.tokenizer = AutoTokenizer.from_pretrained(settings.MODEL_NAME)
        if self.tokenizer.pad_token is None:
            self.tokenizer.pad_token = self.tokenizer.eos_token
        self.base_model = AutoModelForCausalLM.from_pretrained(settings.MODEL_NAME)
        self.adapters: dict[str, PeftModel] = {}

    def load_adapter(self, author: str) -> PeftModel:
        if author not in self.adapters:
            adapter_path = ADAPTERS_DIR / author / "E3_high_rank"
            if not adapter_path.exists():
                adapter_path = ADAPTERS_DIR / author / "E1_default"
            if not adapter_path.exists():
                raise FileNotFoundError(f"Адаптер не найден: {adapter_path}")
            logger.info("Загрузка адаптера: %s", adapter_path)
            self.adapters[author] = PeftModel.from_pretrained(
                self.base_model, str(adapter_path)
            )
            self.adapters[author].eval()
        return self.adapters[author]

    def preload_all(self):
        """Предзагрузка всех доступных адаптеров."""
        if not ADAPTERS_DIR.exists():
            logger.warning("Директория адаптеров не найдена: %s", ADAPTERS_DIR)
            return
        for author_dir in ADAPTERS_DIR.iterdir():
            if author_dir.is_dir() and (author_dir / "E3_high_rank").exists() or (author_dir / "E1_default").exists():
                self.load_adapter(author_dir.name)
        logger.info("Загружено адаптеров: %d", len(self.adapters))
