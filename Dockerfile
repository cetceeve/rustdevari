FROM debian:bullseye-slim

COPY ./target/release/rustdevari-etcd /usr/bin/rustdevari-etcd

EXPOSE 8080

CMD ["rustdevari-etcd"]
