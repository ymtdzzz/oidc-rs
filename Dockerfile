FROM rustlang/rust:nightly

WORKDIR /app

COPY . .
COPY ./private-key.pem /app/src/private-key.pem
RUN cargo build --release

ENTRYPOINT ["/app/target/release/main"]
