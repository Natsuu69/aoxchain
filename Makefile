.PHONY: build test check run-local fmt

build:
	cargo build --workspace

test:
	cargo test --workspace

check:
	cargo check --workspace

fmt:
	cargo fmt --all

run-local:
	./scripts/run-local.sh
