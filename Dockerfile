FROM rust:latest as builder
MAINTAINER Kevin Oberlies <favilo@gmail.com>

RUN cargo install cargo-make
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN rustup default nightly
RUN apt-get update && apt-get install llvm-11 apt-utils libclang-11-dev -y

WORKDIR /app
ADD . ./

RUN cargo +nightly make build_release

FROM debian:bullseye-slim
EXPOSE 8000

RUN apt-get update

WORKDIR /app
RUN mkdir /app/db

COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/backend /app/raspylights
COPY --from=builder /app/frontend/index.html /app/frontend/index.html
COPY --from=builder /app/frontend/pkg/ /app/frontend/pkg
COPY --from=builder /app/frontend/static/ /app/frontend/static

ENV RUST_BACKTRACE=full
ENV RUST_LIB_BACKTRACE=1
CMD ["/app/raspylights"]
