PORT ?= /dev/ttyUSB0
BAUD ?= 460800

.PHONY: all test check build-everything build-minimal flash-everything flash-minimal docs clean help

all: test

## Run workspace unit tests and headless visual regression QA
test:
	cargo test --workspace --verbose

## Verify compilation across all workspace crates
check:
	cargo check --workspace

## Build official Everything example (all 25 widgets)
build-everything:
	idf.py -C examples/everything -DIDF_TARGET=esp32s3 build

## Flash official Everything example to device
flash-everything:
	idf.py -C examples/everything -p $(PORT) -b $(BAUD) flash

## Build official Minimal starter example
build-minimal:
	idf.py -C examples/minimal -DIDF_TARGET=esp32s3 build

## Flash official Minimal starter example
flash-minimal:
	idf.py -C examples/minimal -p $(PORT) -b $(BAUD) flash

## Build ESP-IDF Unity component unit tests (esp_lvgl_bridge & bsp_guition_s3_4848s040)
build-component-tests:
	idf.py -C test_apps/component_tests -DIDF_TARGET=esp32s3 build

## Flash and run ESP-IDF Unity component unit tests on device
flash-component-tests:
	idf.py -C test_apps/component_tests -p $(PORT) -b $(BAUD) flash monitor

## Build mdBook documentation (runs test suite, syncs visual artifacts, and builds book)
docs: test
	@mkdir -p docs/src/artifacts
	@cp -f artifacts/*.png docs/src/artifacts/ 2>/dev/null || true
	@cp -f examples/everything/crates/everything-app/artifacts/*.png docs/src/artifacts/ 2>/dev/null || true
	@cp -f examples/minimal/crates/minimal-app/artifacts/*.png docs/src/artifacts/ 2>/dev/null || true
	mdbook build

## Live serve mdBook documentation on localhost:3000
docs-serve:
	mdbook serve --open

## Clean Cargo build artifacts
clean:
	cargo clean

help:
	@echo "ESP32 Hybrid C/Rust LVGL Framework - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make test              - Run all workspace tests and headless visual QA"
	@echo "  make check             - Run cargo check across all workspace crates"
	@echo "  make build-minimal     - Build examples/minimal firmware"
	@echo "  make flash-minimal     - Flash examples/minimal to device"
	@echo "  make build-everything  - Build examples/everything firmware (all widgets)"
	@echo "  make flash-everything  - Flash examples/everything to device"
	@echo "  make docs              - Build mdBook documentation site (runs tests & syncs visual artifacts)"
	@echo "  make docs-serve        - Live serve mdBook documentation locally"
	@echo "  make clean             - Clean Cargo build targets"

