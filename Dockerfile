FROM rust

COPY . .

RUN cargo install --path --features crash_recovery .

EXPOSE 8080

CMD ["etcd"]
