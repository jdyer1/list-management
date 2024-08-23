FROM rust:1.80.1 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:1.80.1
RUN ldd --version
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get -y install sqlite3 libsqlite3-dev \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get -y install sqlite3 libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/list-management list-management
COPY --from=builder /app/.env .env
ENV SERVER_HOST 0.0.0.0
ENTRYPOINT ["./list-management"]
