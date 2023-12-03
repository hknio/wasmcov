.PHONY: build test

build:
	cargo build --all-features

test:
	export TMPDIR=/tmp
	cargo test --all-features -- --test-threads=1

bin:
	cargo build --all-features --bin cargo-wasmcov
	./target/debug/cargo-wasmcov wasmcov
