build:
	cargo build --release
	sudo docker build -t op-etcd .
run:
	sudo docker-compose up --remove-orphans
all: build run
