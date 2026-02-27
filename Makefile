.PHONY: build dev test lint fmt clean check

build:
	cargo build --release

dev:
	cargo run -- --config config.yaml

test:
	cargo test --workspace

lint:
	cargo fmt --check
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean

check:
	cargo check --workspace
