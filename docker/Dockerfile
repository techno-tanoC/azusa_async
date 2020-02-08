FROM rust:1.41.0-slim-buster

WORKDIR /build

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && \
    apt install -y --no-install-recommends build-essential musl-tools libssl-dev

COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build

COPY . .
RUN cargo build


FROM debian:10.2-slim

WORKDIR /app
RUN apt update && apt install -y ca-certificates
COPY --from=0 /build/target/x86_64-unknown-linux-musl/debug/azusa azusa
