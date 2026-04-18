"""Генерация ответа в стиле автора."""

import torch

from src.generation.model import ModelManager


class AnswerGenerator:
    def __init__(self, model_manager: ModelManager):
        self.model_manager = model_manager
        self.tokenizer = model_manager.tokenizer

    def generate(self, question: str, author: str, context_chunks: list[str]) -> str:
        model = self.model_manager.load_adapter(author)

        context = "\n\n".join(context_chunks)
        prompt = (
            f"Контекст из произведений автора:\n{context}\n\n"
            f"Вопрос читателя: {question}\n\n"
            f"Ответ в стиле автора:"
        )

        inputs = self.tokenizer(
            prompt, return_tensors="pt", truncation=True, max_length=1024
        )
        with torch.no_grad():
            outputs = model.generate(
                **inputs,
                max_new_tokens=256,
                temperature=0.8,
                top_p=0.9,
                repetition_penalty=1.2,
                do_sample=True,
            )

        answer_tokens = outputs[0][inputs["input_ids"].shape[1]:]
        answer = self.tokenizer.decode(answer_tokens, skip_special_tokens=True)
        return answer.strip()
