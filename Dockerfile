FROM rust

COPY . .

RUN cargo install --path .

EXPOSE 8080

CMD ["etcd"]
