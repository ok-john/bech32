
default :: build

show ::
	cat Makefile

build ::
	cargo build --release

test ::
	RUST_BACKTRACE=1 cargo test

.PHONY: show build test
