FROM rust:1.42 AS builder
WORKDIR /usr/src/myapp

COPY Cargo.toml .
COPY Cargo.lock .
COPY ./hashcode_score_calc/Cargo.toml ./hashcode_score_calc/Cargo.toml
COPY ./hashcode_server/Cargo.toml ./hashcode_server/Cargo.toml

RUN cargo fetch

COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl1.1
COPY --from=builder /usr/src/myapp/target/release/hashcode_server /usr/local/bin/myapp
COPY --from=builder /usr/src/myapp/hashcode_server/static /usr/local/bin/static
WORKDIR /usr/local/bin
CMD ["myapp"]
