.PHONY: all test check build-s3 build-minimal build-cookbook docs clean help

all: test

## Run workspace unit tests and headless visual regression QA
test:
	cargo test --workspace --verbose

## Verify compilation across all workspace crates
check:
	cargo check --workspace

## Build official S3 Flagship 480x480 example
build-s3:
	idf.py -C examples/s3_4848_flagship -DIDF_TARGET=esp32s3 build

## Build official Minimal starter example
build-minimal:
	idf.py -C examples/minimal -DIDF_TARGET=esp32s3 build

## Build official Widget Cookbook example
build-cookbook:
	idf.py -C examples/widget_cookbook -DIDF_TARGET=esp32s3 build

## Build mdBook documentation
docs:
	mdbook build

## Clean Cargo build artifacts
clean:
	cargo clean

help:
	@echo "ESP32 Hybrid C/Rust LVGL Framework - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make test            - Run all workspace tests and headless visual QA"
	@echo "  make check           - Run cargo check across all workspace crates"
	@echo "  make build-s3        - Build examples/s3_4848_flagship firmware"
	@echo "  make build-minimal   - Build examples/minimal firmware"
	@echo "  make build-cookbook  - Build examples/widget_cookbook firmware"
	@echo "  make docs            - Build mdBook documentation site"
	@echo "  make clean           - Clean Cargo build targets"
