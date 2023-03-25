build-full:
	docker build -t rustdevari-etcd .
build-dev:
	cargo build --release --features crash_recovery
	docker build -f dev.Dockerfile -t rustdevari-etcd .
build-dev-pl:
	cargo build --features pl
	docker build -f dev.Dockerfile -t rustdevari-etcd .
run:
	docker compose up --remove-orphans -V
dev: build-dev run
full: build-full run
