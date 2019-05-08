FROM alpine:latest

COPY ./run.sh /
COPY ./target/release/chat /app/
RUN mkdir /lib64 && ln -s /lib/libc.musl-x86_64.so.1 /lib64/ld-linux-x86-64.so.2
RUN chmod +x /app/chat

ENTRYPOINT [ "/run.sh" ]