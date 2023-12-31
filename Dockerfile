FROM rustlang/rust:nightly-bookworm-slim AS build

RUN apt-get update -y
RUN apt-get install -y libssl-dev pkg-config

COPY . .
RUN cargo build --release --bin voxov

FROM debian:bookworm-slim

RUN apt-get update -y
RUN apt-get install -y openssl

EXPOSE 8080
COPY --from=build ./target/release/voxov .
CMD ["./voxov"]
