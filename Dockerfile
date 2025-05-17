FROM rust:1.87 as build

RUN cargo new --bin gpm
WORKDIR /gpm

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./sgcp ./sgcp
COPY ./build.rs ./build.rs

RUN apt-get update && apt-get install -y protobuf-compiler

RUN cargo build --release

FROM debian:bookworm-slim 
COPY --from=build /gpm/target/release/gpm .
CMD ["./gpm"]
