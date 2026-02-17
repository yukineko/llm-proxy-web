FROM rust:latest AS builder

WORKDIR /app

# Create dummy project for dependency caching
COPY Cargo.toml ./
RUN mkdir -p src/bin src/indexer src/rag src/filters && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/rag_indexer.rs && \
    echo "" > src/lib.rs && \
    echo "" > src/models.rs && \
    echo "" > src/proxy.rs && \
    echo "" > src/logger.rs && \
    echo "pub mod pii_detector; pub mod output_sanitizer;" > src/filters/mod.rs && \
    echo "pub struct PIIDetector;" > src/filters/pii_detector.rs && \
    echo "pub struct OutputSanitizer;" > src/filters/output_sanitizer.rs && \
    echo "pub mod embeddings; pub mod vector_store; pub mod index_manager; pub mod versioning;" > src/rag/mod.rs && \
    echo "" > src/rag/embeddings.rs && \
    echo "" > src/rag/vector_store.rs && \
    echo "" > src/rag/index_manager.rs && \
    echo "" > src/rag/versioning.rs && \
    echo "pub mod walker; pub mod chunker; pub mod extractor;" > src/indexer/mod.rs && \
    echo "" > src/indexer/walker.rs && \
    echo "" > src/indexer/chunker.rs && \
    echo "" > src/indexer/extractor.rs && \
    cargo build --release --bin llm-proxy 2>/dev/null || true && \
    rm -rf src

# Build actual application
COPY src ./src
RUN find src -name '*.rs' -exec touch {} + && cargo build --release --bin llm-proxy

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN mkdir -p /app/uploads

# Download embedding model files (bge-small-en-v1.5, 384-dim)
RUN mkdir -p /app/models/bge-small-en-v1.5 && \
    curl -L -o /app/models/bge-small-en-v1.5/model.onnx \
      "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/onnx/model.onnx" && \
    curl -L -o /app/models/bge-small-en-v1.5/tokenizer.json \
      "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/tokenizer.json" && \
    curl -L -o /app/models/bge-small-en-v1.5/config.json \
      "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/config.json" && \
    curl -L -o /app/models/bge-small-en-v1.5/special_tokens_map.json \
      "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/special_tokens_map.json" && \
    curl -L -o /app/models/bge-small-en-v1.5/tokenizer_config.json \
      "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/tokenizer_config.json"

COPY --from=builder /app/target/release/llm-proxy /usr/local/bin/llm-proxy

# Add healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD curl -f http://localhost:8080/api/health || exit 1

EXPOSE 8080

CMD ["llm-proxy"]
