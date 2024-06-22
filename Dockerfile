FROM rust:1.79 as build

RUN USER=root cargo new --bin puzzles
WORKDIR /puzzles

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./resources ./resources

RUN rm ./target/release/deps/puzzles*
RUN cargo build --release

FROM rust:1.79-slim

EXPOSE 8080

COPY --from=build /puzzles/target/release/puzzles .

ENTRYPOINT ["./puzzles"]