build-full:
	sudo docker build -t op-etcd .
build-dev:
	cargo build --release --features crash_recovery
	sudo docker build -f DevDockerfile -t op-etcd .
build-dev-pl:
	cargo build --features pl
	sudo docker build -f DevDockerfile -t op-etcd .
run:
	sudo docker-compose up --remove-orphans -V
dev: build-dev run
full: build-full run
