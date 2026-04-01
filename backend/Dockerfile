FROM rust:1.94-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Copy real source and build
COPY . .
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/glyph-backend /usr/local/bin/glyph-backend
COPY --from=builder /app/migrations /app/migrations

WORKDIR /app
EXPOSE 3000
CMD ["glyph-backend"]
