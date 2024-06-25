.PHONY: build test

build:
	cargo build --all-features --release

test:
	export TMPDIR=/tmp
	cargo test --all-features -- --test-threads=1

install:
	cargo install --path .
