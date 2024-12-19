FROM alpine:3.14

WORKDIR /usr/src/jpeg_proxy
COPY ./target/x86_64-unknown-linux-musl/release/jpeg_proxy .

CMD ["/usr/src/jpeg_proxy/jpeg_proxy"]