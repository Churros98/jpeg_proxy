FROM rust:1.67 AS build
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install --path . --target x86_64-unknown-linux-musl

FROM alpine:3.14

WORKDIR /usr/src/jpeg_proxy
COPY --from=build /usr/local/cargo/bin/jpeg_proxy /usr/local/bin/jpeg_proxy

ENTRYPOINT ["/usr/local/bin/jpeg_proxy"]