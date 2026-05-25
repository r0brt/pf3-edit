.PHONY: run test lint fmt check debug-keys

run:
	cargo run -p ispf-tui -- fixtures/sample.txt

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

check: test lint

debug-keys:
	cargo run -p ispf-tui -- --debug-keys
