FROM rust:1.88 as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY .sqlx .sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release 

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/wildlife-api /app/app

CMD ["./app"]