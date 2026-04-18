from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    QDRANT_HOST: str = "qdrant"
    QDRANT_PORT: int = 6333
    MODEL_NAME: str = "ai-forever/rugpt3small_based_on_gpt2"
    EMBEDDING_MODEL: str = "intfloat/multilingual-e5-base"
    TOP_K: int = 5
    CHUNK_SIZE: int = 256
    CHUNK_OVERLAP: int = 64

    class Config:
        env_file = ".env"


settings = Settings()
