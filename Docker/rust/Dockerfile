# builder
FROM rust:1.88.0 AS builder

WORKDIR /usr/src/app

COPY Cargo.toml ./
COPY blog-driver/Cargo.toml blog-driver/
COPY blog-app/Cargo.toml blog-app/
COPY blog-domain/Cargo.toml blog-domain/
COPY blog-adapter/Cargo.toml blog-adapter/

RUN mkdir -p blog-driver/src && echo "fn main() {println!(\"Hello, world!\")}" > blog-driver/src/main.rs &&\
    mkdir -p blog-app/src && echo "fn main() {println!(\"Hello, world!\")}" > blog-app/src/main.rs &&\
    mkdir -p blog-domain/src && echo "fn main() {println!(\"Hello, world!\")}" > blog-domain/src/main.rs &&\
    mkdir -p blog-adapter/src && echo "fn main() {println!(\"Hello, world!\")}" > blog-adapter/src/main.rs

RUN cargo build --release --color never &&\
    rm blog-driver/src/main.rs &&\
    rm blog-app/src/main.rs &&\
    rm blog-domain/src/main.rs &&\
    rm blog-adapter/src/main.rs

COPY blog-driver/ blog-driver/
COPY blog-app/ blog-app/
COPY blog-domain/ blog-domain/
COPY blog-adapter blog-adapter/

RUN cargo build --release --color never

# stg
FROM debian:bookworm-slim AS stg

RUN apt-get update &&\
    apt install -y openssl ca-certificates &&\
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/blog-driver /usr/local/bin/

CMD ["blog-driver"]

# prd
FROM debian:bookworm-slim AS prd

RUN apt-get update &&\
    apt install -y openssl ca-certificates &&\
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/blog-driver /usr/local/bin/

CMD ["blog-driver"]
