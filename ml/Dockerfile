FROM python:3.11-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends gcc && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir --upgrade pip

# CPU-only torch (200 MB вместо 3 GB с CUDA)
RUN pip install --no-cache-dir torch --index-url https://download.pytorch.org/whl/cpu

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY src/ ./src/
COPY scripts/ ./scripts/
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8001"]
