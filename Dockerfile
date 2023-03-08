FROM ubuntu

COPY ./target/release/etcd /usr/bin/etcd

RUN apt-get update && apt-get -y install wget

RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb

RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2.17_amd64.deb

EXPOSE 8080

CMD ["etcd"]
