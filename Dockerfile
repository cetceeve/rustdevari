FROM rust

COPY . .

RUN cargo install --features crash_recovery --path .

EXPOSE 8080

CMD ["etcd"]
