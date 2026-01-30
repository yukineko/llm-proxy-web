FROM rust:1.75 as builder

WORKDIR /app

# Dependencies cache layer
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build actual application
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/llm-proxy /usr/local/bin/llm-proxy

EXPOSE 8080

CMD ["llm-proxy"]
