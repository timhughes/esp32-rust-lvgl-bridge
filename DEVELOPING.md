# DEVELOPING.md — Developer & Contributor Guide

Welcome to the development guide for the **ESP32 Hybrid C/Rust LVGL Framework**.

This document is intended for engineers developing, testing, maintaining, and extending the framework codebase itself (its components, crates, test harnesses, and examples).

> [!NOTE]
> If you are an AI assistant or autonomous coding agent working in this repository, please also read [`AGENTS.md`](AGENTS.md) for agent-specific rules, architectural boundaries, and workflow procedures.

---

## 1. System Architecture & Boundaries

The central design principle of this repository is:

> **"C owns the display; Rust owns the system."**

```
┌────────────────────────────────────────────────────────────────────────┐
│                          ESP32 Dual-Core SoC                           │
│                                                                        │
│   CORE 1 (C / ESP-IDF)                      CORE 0 (Rust `no_std`)     │
│  ┌───────────────────────┐                 ┌────────────────────────┐  │
│  │   LVGL v9 Scene Graph │                 │ Domain State Machine   │  │
│  │   ST7701S RGB Driver  │   FreeRTOS      │ Closed-Loop Control    │  │
│  │   GT911 Touch I2C     │   Queues / POD  │ Sensor Filter / Math   │  │
│  │   Backlight Control   │ ◄─────────────► │ Protocol Decoders      │  │
│  │   Temporal Holdoff    │   (Zero-Copy)   │ Deterministic Workers  │  │
│  └───────────────────────┘                 └────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

### Separation of Responsibilities

1. **C / ESP-IDF Layer ([`components/esp_lvgl_bridge`](components/esp_lvgl_bridge))**:
   - Manages hardware drivers (ST7701S 16-bit RGB parallel bus, GT911 touch digitizer over I2C).
   - Manages FreeRTOS task pinning (Core 1 for LVGL render and touch loops, Core 0 for Rust domain loop).
   - Dynamically parses the declarative UI schema tables and constructs native LVGL v9 widget objects.
   - Enforces the 3-stage arbitration engine (`ARB_TRACKING`, `ARB_DRAGGING`, `ARB_HOLDOFF`) and inactivity backlight timeout.

2. **Rust `no_std` Layer ([`crates/esp32-rust-lvgl-bridge-core`](crates/esp32-rust-lvgl-bridge-core))**:
   - Compiles as a bare-metal static library (`staticlib`) with `#![no_std]`.
   - Defines the ABI data types (`ParamId`, `ParamVal`, `UiCommand`, `DeviceTelemetry`, `WidgetDescriptor`).
   - Implements closed-loop state machines, control logic, telemetry generators, and declarative UI schema definitions.
   - **Zero heap allocations** across the FFI boundary; **zero raw pointers** to LVGL objects.

3. **FFI Transport Boundary**:
   - Strict Plain Old Data (POD) messaging via FreeRTOS ring queues on target hardware, or ring buffers in the host test harness.
   - Messages are copied by value (`sizeof(UiCommand)` = 40 bytes, `sizeof(DeviceTelemetry)` = 40 bytes).

---

## 2. Repository Layout

```text
esp32-rust-lvgl-framework/
├── README.md                           # User-facing guide for building applications
├── DEVELOPING.md                       # This developer & contributor manual
├── AGENTS.md                           # AI agent guidelines & workflow rules
├── Makefile                            # Standard automation tasks and shortcuts
├── Cargo.toml                          # Rust workspace definition
├── components/                         # Reusable ESP-IDF components
│   ├── esp_lvgl_bridge/                # Core C transport & UI engine component
│   │   ├── include/                    # Public C headers (bridge_protocol.h, ui_binding.h, etc.)
│   │   ├── src/                        # Implementations (ui_binding.c, bridge_transport.c, etc.)
│   │   ├── test/                       # ESP-IDF Unity component unit tests
│   │   └── cmake/esp_rust.cmake        # CMake macro to compile & link Rust staticlibs
│   └── bsp_guition_s3_4848s040/        # Board Support Package for Guition 4848S040
├── crates/                             # Reusable Rust crates
│   ├── esp32-rust-lvgl-bridge-core/    # no_std ABI contracts, schema descriptors & types
│   └── esp32-rust-lvgl-bridge-harness/ # Headless LVGL host test harness (pure Rust + C FFI)
├── examples/                           # Official reference applications
│   ├── minimal/                        # Minimal starter template (minimal-app crate)
│   └── everything/                     # Full 34-widget reference showcase (everything-app crate)
├── test_apps/                          # Target-level test applications
│   └── component_tests/                # ESP-IDF Unity runner app for physical/QEMU testing
└── docs/                               # mdBook documentation source and static assets
```

---

## 3. Development Environment Setup

### Host Development (Linux / macOS)
To run unit tests, compile headless LVGL simulations, and generate documentation locally:
- **Rust**: Nightly or stable toolchain:
  ```bash
  rustup default nightly
  rustup component add rust-src
  ```
- **C Compiler**: `gcc` or `clang`, `make`, `pkg-config`.
- **mdBook** (optional, for building documentation):
  ```bash
  cargo install mdbook
  ```

### Target Development (ESP-IDF)
To build and flash firmware to physical ESP32 hardware:
- **ESP-IDF**: Version 5.4 or 5.5.
- **Python**: 3.10+ (Python 3.13 supported):
  ```bash
  # Source ESP-IDF environment
  source $HOME/esp/esp-idf/export.sh
  ```
- **Rust Xtensa Target**: Configured automatically via `.cargo/config.toml` using `xtensa-esp32s3-none-elf`.

---

## 4. Common Developer Workflows

### Running Tests Locally
```bash
# Run all workspace unit tests and headless visual regression suites:
cargo test --workspace

# Or use the Makefile:
make test
```

### Verifying Compilation Across the Workspace
```bash
make check
```

### Building ESP-IDF Component Unit Tests
```bash
make build-component-tests
```
To flash and run the Unity test suite on hardware:
```bash
make flash-component-tests PORT=/dev/ttyUSB0
```

### Building & Flashing Example Applications
```bash
# Minimal Starter:
make build-minimal
make flash-minimal PORT=/dev/ttyUSB0

# Everything Showcase (all 34 widgets, 7 tabs):
make build-everything
make flash-everything PORT=/dev/ttyUSB0
```

### Generating Documentation
```bash
# Re-run visual tests, sync rendered PNGs, and build the mdBook:
make docs

# Live serve documentation on http://localhost:3000:
make docs-serve
```

---

## 5. Coding Standards & Guidelines

### C Code (`components/esp_lvgl_bridge`)
1. **Preprocessor Guards (`#if LV_USE_*`)**:
   - Optional or modular LVGL widgets (e.g. `LV_USE_ARCLABEL`, `LV_USE_QRCODE`, `LV_USE_WIN`, `LV_USE_IMAGEBUTTON`, `LV_USE_SPAN`, `LV_USE_ANIMIMG`, `LV_USE_BARCODE`, `LV_USE_FLEX`) **must be wrapped in `#if LV_USE_*`** across type declarations, static arrays, callback definitions, and schema construction cases.
   - This ensures minimal SDK configurations compile cleanly without `-Wunused-variable` warnings.
2. **Warning-Free Compilation**:
   - All C code must compile with `-Wall -Wextra -Werror` compliance under both host Clang/GCC and target Xtensa GCC.
   - For deprecated LVGL functions in test cases or migration shims, use localized `#pragma GCC diagnostic push / ignored "-Wdeprecated-declarations" / pop` blocks.
3. **No Dynamic Heap in Render Loops**:
   - Scene graph memory is allocated at schema initialization time.
   - FreeRTOS queues pass fixed-size POD buffers; do not allocate heap memory per frame or per telemetry tick.

### Rust Code (`crates/` and `examples/*/crates/`)
1. **`no_std` Compliance**:
   - `esp32-rust-lvgl-bridge-core` and application crates must be strictly `#![no_std]` when targeting bare-metal microcontrollers (`cfg_attr(target_os = "none", no_std)`).
2. **ABI Synchronization**:
   - Any changes to struct fields or layout in [`crates/esp32-rust-lvgl-bridge-core/src/abi.rs`](crates/esp32-rust-lvgl-bridge-core/src/abi.rs) **must be mirrored exactly** in [`components/esp_lvgl_bridge/include/bridge_protocol.h`](components/esp_lvgl_bridge/include/bridge_protocol.h).
   - Use `#[repr(C)]` for all types crossing the FFI boundary.
3. **Open Parameter Architecture**:
   - Use the `ParamId` newtype for all parameters. Do not introduce monolithic enums into the shared core crate. Applications define their own domain constants using `ParamId::new(X)`.

---

## 6. Testing Strategy

1. **Host-Native Headless Visual QA**:
   - Implemented via [`crates/esp32-rust-lvgl-bridge-harness`](crates/esp32-rust-lvgl-bridge-harness).
   - Runs offscreen in software raster mode (no GPU, no X11/Wayland required).
   - Captures rendered framebuffers to PNG and performs pixel-perfect visual assertions.
   - Injects touch events (`click`, `drag_slider`, `touch_set`, `touch_release`) and steps simulated time (`step(ms)`).

2. **C Component Integration Tests**:
   - Implemented in [`crates/esp32-rust-lvgl-bridge-harness/tests/c_bridge_components_test.rs`](crates/esp32-rust-lvgl-bridge-harness/tests/c_bridge_components_test.rs) (host FFI) and [`components/esp_lvgl_bridge/test/`](components/esp_lvgl_bridge/test) (target Unity).
   - Exercises transport ring buffers, null pointer guards, boundary lookups, P4-C6 packet framing, and schema rebuild cycles.

3. **On-Target Unity Test Runner**:
   - Contained in [`test_apps/component_tests`](test_apps/component_tests).
   - Validates execution directly on Xtensa LX7 and RISC-V hardware.

---

## 7. Adding a New Widget Type (Step-by-Step)

To add support for a new LVGL widget type to the framework:

1. **Define the ABI Enum Variant**:
   - In [`crates/esp32-rust-lvgl-bridge-core/src/abi.rs`](crates/esp32-rust-lvgl-bridge-core/src/abi.rs), add the new variant to `WidgetType`.
   - Add a builder constructor on `WidgetDescriptor` (e.g. `WidgetDescriptor::my_widget(...)`).

2. **Mirror in C Header**:
   - In [`components/esp_lvgl_bridge/include/bridge_protocol.h`](components/esp_lvgl_bridge/include/bridge_protocol.h), add the matching entry to `widget_type_t`.

3. **Implement in C Binding Engine**:
   - In [`components/esp_lvgl_bridge/src/ui_binding.c`](components/esp_lvgl_bridge/src/ui_binding.c):
     - Wrap any static tracking array or struct in `#if LV_USE_MY_WIDGET`.
     - Add the construction branch to `ui_engine_build_from_schema`.
     - Add telemetry dispatch logic to `ui_engine_dispatch_telemetry` if the widget accepts live updates.
     - Add event callback handlers if the widget accepts user interaction.

4. **Add Unit Tests**:
   - Add target test cases in [`components/esp_lvgl_bridge/test/test_ui_binding.c`](components/esp_lvgl_bridge/test/test_ui_binding.c).
   - Add host test cases in [`crates/esp32-rust-lvgl-bridge-harness/tests/c_bridge_components_test.rs`](crates/esp32-rust-lvgl-bridge-harness/tests/c_bridge_components_test.rs).

5. **Expose in Example & Verify**:
   - Add the widget to [`examples/everything/crates/everything-app/src/lib.rs`](examples/everything/crates/everything-app/src/lib.rs).
   - Run `cargo test --workspace` and `make build-component-tests` to ensure clean verification.
