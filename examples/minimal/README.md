# Minimal Starter Example: ESP32 Rust + LVGL

This example demonstrates how to build a clean, modular application using the **ESP32 Rust/LVGL Framework**.

It sets up:
* **Custom Parameter IDs:** Declares application-specific parameter IDs without modifying the core library.
* **3-Widget Declarative Schema:** A Power Switch, a Speed Slider, and a Cycle Counter Label.
* **Asymmetric Dual-Core Architecture:** FreeRTOS tasks partitioned across Core 0 (Rust domain worker) and Core 1 (LVGL display).

---

## Architecture Overview

```text
  CORE 0 (Rust no_std)                    CORE 1 (C / LVGL v9)
 ┌─────────────────────────┐             ┌─────────────────────────┐
 │ MinimalEngine           │   FreeRTOS  │ UI Scene Graph          │
 │  - PARAM_POWER   (101)  │    Queues   │  - Switch (TopMid)      │
 │  - PARAM_SPEED   (102)  │ ◄─────────► │  - Slider (Center)      │
 │  - PARAM_COUNTER (103)  │             │  - Label  (BottomMid)   │
 └─────────────────────────┘             └─────────────────────────┘
```

---

## Project Structure

```text
examples/minimal/
├── CMakeLists.txt              # Standalone ESP-IDF project configuration
├── README.md                   # This guide
├── main/
│   ├── CMakeLists.txt          # Links esp_lvgl_bridge, BSP, and Rust
│   └── main.c                  # C task initialization & scene graph construction
└── crates/
    └── minimal-app/            # no_std Rust application logic
        ├── Cargo.toml
        └── src/lib.rs          # Parameter IDs, declarative schema & state machine
```

---

## Quickstart

### 1. Build the Rust Staticlib
```bash
cargo build --release --target xtensa-esp32s3-none-elf -p minimal-app
```

### 2. Build and Flash the Firmware
```bash
cd examples/minimal
idf.py build
idf.py -p /dev/ttyUSB0 flash monitor
```
