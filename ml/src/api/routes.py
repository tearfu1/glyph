"""API маршруты ML-сервиса."""

import logging

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

logger = logging.getLogger(__name__)

router = APIRouter()


# Карта соответствия русских имён авторов в полный slug, который используется
# в Qdrant payload и названиях каталогов адаптеров. Бэкенд шлёт полное имя
# автора из таблицы up_user — здесь приводим его к slug.
AUTHOR_ALIASES: dict[str, str] = {
    "достоевский": "dostoevsky",
    "dostoevsky": "dostoevsky",
    "чехов": "chekhov",
    "chekhov": "chekhov",
    "булгаков": "bulgakov",
    "bulgakov": "bulgakov",
}


def resolve_author_slug(raw: str) -> str | None:
    """Возвращает slug автора по произвольному имени или None."""
    if not raw:
        return None
    name = raw.strip().lower()
    if name in AUTHOR_ALIASES:
        return AUTHOR_ALIASES[name]
    # Частое: полное имя вида "Фёдор Михайлович Достоевский" — ищем фамилию
    for alias, slug in AUTHOR_ALIASES.items():
        if alias in name:
            return slug
    return None


class GenerateRequest(BaseModel):
    question: str
    author: str
    # book_id — UUID-строка от Rust-бэкенда; сейчас не используется,
    # оставлено для будущей фильтрации по конкретной книге
    book_id: str | None = None


class SourceChunk(BaseModel):
    text: str
    score: float
    book: str


class GenerateResponse(BaseModel):
    answer: str
    sources: list[SourceChunk]


@router.post("/generate-answer", response_model=GenerateResponse)
async def generate_answer(request: GenerateRequest):
    from src.main import retriever, generator

    if retriever is None or generator is None:
        raise HTTPException(status_code=503, detail="ML модели не загружены")

    author_slug = resolve_author_slug(request.author)
    if author_slug is None:
        raise HTTPException(
            status_code=400,
            detail=f"Неизвестный автор: '{request.author}'. Поддерживаются: {', '.join(set(AUTHOR_ALIASES.values()))}",
        )

    try:
        chunks = retriever.retrieve(request.question, author_slug)
        if not chunks:
            raise HTTPException(status_code=404, detail=f"Нет данных для автора: {author_slug}")

        answer = generator.generate(
            request.question, author_slug, [c["text"] for c in chunks]
        )

        sources = [
            SourceChunk(text=c["text"], score=c["score"], book=c["book"])
            for c in chunks
        ]
        return GenerateResponse(answer=answer, sources=sources)

    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        logger.exception("Ошибка генерации")
        raise HTTPException(status_code=500, detail=str(e))
