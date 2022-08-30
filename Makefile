
default :: build

show ::
	cat Makefile

fmt ::
	rustfmt src/lib.rs

build :: fmt
	cargo build --release

test ::
	RUST_BACKTRACE=1 cargo test

.PHONY: show build test
