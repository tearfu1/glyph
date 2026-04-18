"""FastAPI приложение Glyph ML Service."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI

from src.api.routes import router
from src.config import settings

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

retriever = None
generator = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global retriever, generator

    try:
        from src.rag.embedder import Embedder
        from src.rag.retriever import Retriever
        from src.generation.model import ModelManager
        from src.generation.generator import AnswerGenerator

        logger.info("Загрузка ML моделей...")
        embedder = Embedder()
        retriever = Retriever(embedder)

        model_manager = ModelManager()
        model_manager.preload_all()
        generator = AnswerGenerator(model_manager)

        logger.info("ML модели загружены")
    except Exception as e:
        logger.warning("ML модели не загружены (сервис работает без генерации): %s", e)

    yield

    retriever = None
    generator = None


app = FastAPI(title="Glyph ML Service", lifespan=lifespan)
app.include_router(router, prefix="/api")


@app.get("/health")
def health():
    return {
        "status": "ok",
        "models_loaded": generator is not None,
    }
