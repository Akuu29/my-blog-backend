FROM rust:latest

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

RUN cargo install cargo-watch

CMD ["cargo", "watch", "-x", "run"]
