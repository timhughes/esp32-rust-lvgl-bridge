# ESP32 Hybrid C/Rust LVGL Framework

[![Architecture: C + Rust](https://img.shields.io/badge/Architecture-C%20%2B%20Rust%20(no__std)-blue.svg)](#the-core-philosophy)
[![LVGL: v9](https://img.shields.io/badge/LVGL-v9.4-orange.svg)](https://lvgl.io)
[![ESP-IDF: v5.4+](https://img.shields.io/badge/ESP--IDF-v5.4%2B-red.svg)](https://github.com/espressif/esp-idf)
[![Target: S3 | P4 | C6](https://img.shields.io/badge/Targets-ESP32--S3%20|%20P4%20|%20C6-green.svg)](#supported-hardware)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-brightgreen.svg)](https://timhughes.github.io/esp32-rust-lvgl-bridge/)

A production-ready embedded firmware framework and automated visual QA testing pipeline for modern Espressif microcontrollers (**ESP32-S3**, **ESP32-P4**, **ESP32-C6**).

Combines the industry-standard **LVGL v9** graphics engine in C with safe, deterministic **`no_std` Rust** for domain logic, closed-loop state machines, and protocols.

---

## The Core Philosophy

> **"C owns the display; Rust owns the system."**

```text
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

1. **C / ESP-IDF (Display & Hardware Layer):** Drives display peripherals, DMA framebuffers, touch digitizers, FreeRTOS tasks, and the LVGL v9 scene graph.
2. **Rust `no_std` (Application Core):** Implements all domain logic, closed-loop state machines, protocols, and data models inside an isolated `staticlib`.
3. **POD FFI Boundary:** Communication occurs strictly via value-typed FreeRTOS ring queues passing fixed-size Plain Old Data (POD) structs (`UiCommand`, `DeviceTelemetry`). Zero shared heap memory, zero raw `lv_obj_t*` pointers crossing into Rust.

---

## Documentation

Full architectural documentation, hardware guides, and API specifications are available online or locally:

* **Online Documentation:** [timhughes.github.io/esp32-rust-lvgl-bridge](https://timhughes.github.io/esp32-rust-lvgl-bridge/)
* **Offline mdBook Source:** [`docs/src/`](docs/src/) (serve locally via `make docs-serve`)
* **Developer & Contributor Manual:** [`DEVELOPING.md`](DEVELOPING.md) — build targets, coding standards, and how to extend the framework
* **AI Agent Instructions:** [`AGENTS.md`](AGENTS.md) — architectural boundaries and workflows for autonomous coding agents

---

## Key Highlights

* **100% Declarative UI Schemas in Rust:** Declare widgets, layouts, parameters, and labels in static Rust tables. The C engine dynamically builds and binds the scene graph in a single call.
* **34 Core LVGL Widgets:** Out-of-the-box support for switches, sliders, dials, gauges, keyboards, text inputs, streaming charts, tables, calendar, canvas, menus, tileviews, media assets, windows, barcodes, and QR codes.
* **Jitter-Free 3-Stage Arbitration:** Eliminates visual rubber-banding during touch interaction via monotonic sequence tickets (`seq_id`) and temporal holdoff leases (`ARB_DRAGGING` $\rightarrow$ `ARB_HOLDOFF` $\rightarrow$ `ARB_TRACKING`).
* **Headless Visual QA (CI/CD):** Render pixel-perfect offscreen screenshots of the entire UI and verify state transitions on Linux/macOS in milliseconds without hardware attached.
* **Turnkey Guition ESP32-S3-4848S040 Support:** Complete board support package for 4.0-inch 480×480 IPS display with automatic 60-second backlight timeout and wake-on-touch.

---

## Quickstart

### Option A: Run Headless Simulation on Host (No Hardware Needed)
Compile and test the full LVGL engine, Rust state machine, and visual regression suites on your workstation in seconds:

```bash
git clone https://github.com/timhughes/esp32-rust-lvgl-framework.git
cd esp32-rust-lvgl-framework

# Run all 54+ workspace unit tests and offscreen visual QA:
cargo test --workspace
# Or:
make test
```

### Option B: Build & Flash to Physical Hardware (ESP32-S3)
Deploy the full 34-widget reference showcase to a **Guition ESP32-S3-4848S040** display (or any ESP32-S3):

```bash
# 1. Source ESP-IDF environment
source $HOME/esp/esp-idf/export.sh

# 2. Build the Rust static library for Xtensa bare-metal
cargo build --release --target xtensa-esp32s3-none-elf -p everything-app

# 3. Build and flash the firmware
make build-everything
make flash-everything PORT=/dev/ttyUSB0
```

---

## Declarative UI in 10 Seconds

Define your user interface in pure static Rust without procedural C boilerplate:

```rust
pub static MY_UI_SCHEMA: &[WidgetDescriptor] = &[
    // Power Toggle Switch
    WidgetDescriptor::switch(
        PARAM_POWER, UiAlign::TopMid, 0, 40,
        c"Power: ON", c"Power: OFF",
    ),
    // Speed Slider with 500ms arbitration holdoff lease
    WidgetDescriptor::slider(
        PARAM_SPEED, UiAlign::Center, 0, -10,
        0, 100, 25, 500,
        c"Speed: %ld RPM (Setting)", c"Speed: %ld RPM (Actual)",
    ),
    // Live Telemetry Readout Label
    WidgetDescriptor::label(
        PARAM_COUNTER, UiAlign::BottomMid, 0, -40,
        c"Cycles: %ld",
    ),
];
```

> **Zero C UI Code:** All widgets, layouts, holdoffs, and telemetry bindings are declared 100% in Rust. The underlying C runtime acts purely as a turnkey display runner (`ui_engine_build_from_schema(schema, count)`), so you never write procedural LVGL C code or touch `lv_obj_t*` pointers.

See [`examples/minimal/`](examples/minimal) for the complete 3-widget starter template, and [`examples/everything/`](examples/everything) for the full 7-tab dashboard.

---

## Supported Hardware

| Target | Architecture | Display & Bus | Primary Role |
|---|---|---|---|
| **Guition 4848S040** | ESP32-S3 Dual Xtensa LX7 (240MHz, 8MB PSRAM, 16MB Flash) | 4.0" 480×480 ST7701S RGB Parallel + GT911 Touch | Turnkey Smart Display |
| **ESP32-P4** | Dual RISC-V HP (400MHz, MIPI-DSI, HW 2D PPA) | High-definition MIPI-DSI Displays | High-Performance GUI Host |
| **ESP32-C6** | Single RISC-V LP (160MHz, Wi-Fi 6, BLE 5.3, 802.15.4) | SDIO 3.0 / High-Speed SPI link to P4 | Wireless Co-Processor |
| **Host PC** | Native Linux / macOS (`x86_64` / `aarch64`) | Offscreen RGB888 memory buffer | Visual QA & Automated Testing |

---

## License

- Framework codebase: Licensed under Apache-2.0 or MIT.
- [LVGL](https://lvgl.io): Licensed under MIT.
- `stb_image_write.h`: Public Domain / MIT.
