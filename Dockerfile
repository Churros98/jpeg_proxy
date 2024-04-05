FROM alpine:3.14

WORKDIR /usr/src/myapp
COPY . .

CMD ["video_proxy"]