.DEFAULT_GOAL := default
compile:
	cargo build --release --features crash_recovery
compile-pl:
	cargo build --release --features pl
build:
	docker build -t rustdevari-etcd .
run:
	docker compose up --remove-orphans -V
default: build run
from-source: compile build run
from-source-pl: compile-pl build run
