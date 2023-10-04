FROM rust:latest

ENV RUST_BACKTRACE=1

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

CMD ["cargo", "test"]