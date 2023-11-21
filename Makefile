.PHONY: build test

build:
	cargo build --all-features

test:
	export TMPDIR=/tmp
	cargo test --all-features -- --test-threads=1
