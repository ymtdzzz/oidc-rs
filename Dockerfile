FROM rustlang/rust:nightly

WORKDIR /app

COPY . .
COPY ./private-key.pem /app/src/private-key.pem
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release

ENTRYPOINT ["/app/target/release/main"]
