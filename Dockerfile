FROM rust:1.80-slim-bookworm as builder
WORKDIR /usr/src/app
COPY . .
# We use mold linker for faster builds if needed, but standard cargo is fine
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/fluxgate /app/fluxgate
COPY --from=builder /usr/src/app/plugins /app/plugins

EXPOSE 8080 8081 8082 9090
ENV RUST_LOG="info"
CMD ["./fluxgate"]
