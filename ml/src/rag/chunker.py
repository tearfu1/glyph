"""Разбиение текста на чанки по токенам с перекрытием."""

from tokenizers import Tokenizer


def chunk_text(
    text: str,
    tokenizer: Tokenizer,
    chunk_size: int = 256,
    overlap: int = 64,
) -> list[dict]:
    """Разбивает текст на чанки фиксированного размера в токенах.

    Returns:
        Список словарей с ключами: text, start_token, end_token.
    """
    encoding = tokenizer.encode(text)
    token_ids = encoding.ids
    chunks = []
    start = 0
    while start < len(token_ids):
        end = min(start + chunk_size, len(token_ids))
        chunk_ids = token_ids[start:end]
        chunk_text = tokenizer.decode(chunk_ids).strip()
        chunks.append({
            "text": chunk_text,
            "start_token": start,
            "end_token": end,
        })
        if end >= len(token_ids):
            break
        start += chunk_size - overlap
    return chunks
