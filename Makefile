.PHONY: build build-release package-bin test check fmt run-local

build:
	cargo build --workspace

build-release:
	cargo build --release -p aoxcmd --bin aoxc

package-bin: build-release
	mkdir -p bin
	cp target/release/aoxc bin/aoxc
	chmod +x bin/aoxc
	@echo "Packaged binary: ./bin/aoxc"

test:
	cargo test --workspace

check:
	cargo check --workspace

fmt:
	cargo fmt --all

run-local: package-bin
	./scripts/run-local.sh
