"""Генерация ответа в стиле автора."""

import torch

from src.generation.model import ModelManager


PROMPT_TEMPLATE = (
    "Ты — литературный критик, отвечающий строго на основе приведённых фрагментов.\n"
    "Если в фрагментах нет прямого ответа — скажи, что в контексте ответа нет.\n"
    "Не придумывай факты, имена, даты, не относящиеся к фрагментам.\n\n"
    "Фрагменты из произведений автора:\n{context}\n\n"
    "Вопрос читателя: {question}\n\n"
    "Краткий ответ (один абзац) в стиле автора, опираясь ТОЛЬКО на фрагменты выше:"
)


def build_prompt(question: str, context_chunks: list[str]) -> str:
    context = "\n\n".join(context_chunks)
    return PROMPT_TEMPLATE.format(context=context, question=question)


class AnswerGenerator:
    def __init__(self, model_manager: ModelManager):
        self.model_manager = model_manager
        self.tokenizer = model_manager.tokenizer

    def generate(self, question: str, author: str, context_chunks: list[str]) -> str:
        model = self.model_manager.load_adapter(author)
        prompt = build_prompt(question, context_chunks)

        inputs = self.tokenizer(
            prompt, return_tensors="pt", truncation=True, max_length=1024
        )
        with torch.no_grad():
            outputs = model.generate(
                **inputs,
                max_new_tokens=200,
                temperature=0.3,
                top_p=0.7,
                top_k=40,
                repetition_penalty=1.4,
                no_repeat_ngram_size=3,
                do_sample=True,
                pad_token_id=self.tokenizer.eos_token_id,
            )

        answer_tokens = outputs[0][inputs["input_ids"].shape[1]:]
        answer = self.tokenizer.decode(answer_tokens, skip_special_tokens=True)
        return answer.strip()
