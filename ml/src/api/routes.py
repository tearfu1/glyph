"""API маршруты ML-сервиса."""

import logging

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

logger = logging.getLogger(__name__)

router = APIRouter()


class GenerateRequest(BaseModel):
    question: str
    author: str
    book_id: int | None = None


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

    try:
        chunks = retriever.retrieve(request.question, request.author)
        if not chunks:
            raise HTTPException(status_code=404, detail=f"Нет данных для автора: {request.author}")

        answer = generator.generate(
            request.question, request.author, [c["text"] for c in chunks]
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
