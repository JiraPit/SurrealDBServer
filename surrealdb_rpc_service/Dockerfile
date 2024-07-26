FROM rust:1.79 as builder

RUN apt-get update && \
  apt-get install -y --no-install-recommends  \
  ca-certificates \
  libssl-dev \
  libprotobuf-dev \
  protobuf-compiler 
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY dependencies /build/dependencies
COPY src /build/src
COPY Cargo.toml /build/Cargo.toml
COPY "proto(copied)" /build/proto
COPY build_docker.rs /build/build.rs

RUN cargo build --release

FROM debian:bookworm-slim as runtime

RUN apt-get update && \ 
  apt-get install -y --no-install-recommends \
  ca-certificates \
  libssl-dev \
  libprotobuf-dev \
  protobuf-compiler \
  curl
RUN rm -rf /var/lib/apt/lists/*

RUN curl -sSf https://install.surrealdb.com | sh
RUN surreal help

WORKDIR /app

COPY --from=builder /build/target/release/surrealdb_rpc_service /app/surrealdb_rpc_service
COPY --from=builder /build/dependencies /app/dependencies

EXPOSE 50051
EXPOSE 9000
CMD ["./surrealdb_rpc_service"]

