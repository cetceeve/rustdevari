build-full:
	sudo docker build -t op-etcd .
build-dev:
	cargo build --release
	sudo docker build -f DevDockerfile -t op-etcd .
run:
	sudo docker-compose up --remove-orphans
dev: build-dev run
full: build-full run
