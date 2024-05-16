FROM alpine:3.14

WORKDIR /usr/src/myapp
COPY ./target/x86_64-unknown-linux-musl/release/voiturerc_proxy .

CMD ["voiturerc_proxy"]