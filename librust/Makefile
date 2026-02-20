ARCH_RUST_TARGET ?= i686-unknown-linux-gnu

.PHONY: build release test clean run help

help:
	@echo "Available targets:"
	@echo "  build       - Build the project in debug mode"
	@echo "  release     - Build the project in release mode"
	@echo "  test        - Run all tests"
	@echo "  clean       - Remove build artifacts"
	@echo "  run         - Run the project"
	@echo "  check       - Check the project without building"
	@echo "  fmt         - Format the code"
	@echo "  lint        - Run clippy linter"

build:
	cargo build --release --target=$(ARCH_RUST_TARGET)

release:
	cargo build --release --target=$(ARCH_RUST_TARGET)

test:
	cargo test

clean:
	cargo clean

run:
	cargo run

check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

all: clean build test