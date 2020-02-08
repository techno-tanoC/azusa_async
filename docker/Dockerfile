FROM ubuntu:18.04

WORKDIR /build

RUN apt update && apt install -y curl build-essential musl-tools libssl-dev ca-certificates
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly --target x86_64-unknown-linux-musl
ENV PATH=/root/.cargo/bin/:$PATH

COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build

COPY . .
RUN cargo build --out-dir out -Z unstable-options


FROM ubuntu:18.04

WORKDIR /app
RUN apt update && apt install -y ca-certificates
COPY --from=0 /build/out/ ./
