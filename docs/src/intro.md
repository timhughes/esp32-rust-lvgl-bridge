# Introduction

The **ESP32 Hybrid C/Rust LVGL Framework** is a multi-target embedded firmware architecture and automated testing pipeline designed for modern Espressif microcontrollers (ESP32-S3, ESP32-P4, and ESP32-C6).

## The Core Philosophy

> **"C owns the display; Rust owns the system."**

* **C & ESP-IDF**:
  * Drives display peripherals (ST7701S RGB parallel bus, MIPI-DSI, DMA buffers).
  * Manages touch input digitizers (Goodix GT911).
  * Hosts the LVGL v9 immediate/retained UI scene graph and manages GPU/PPA hardware acceleration.
* **Rust (`no_std`)**:
  * Implements all deterministic business logic, state machines, protocol decoders, and device drivers.
  * Guarantees memory safety and thread safety without dynamic heap allocations.
* **Decoupled FFI Boundary**:
  * No raw C pointers, heap references, or `lv_obj_t*` types cross into Rust.
  * Inter-task synchronization uses FreeRTOS message queues passing Plain Old Data (POD) structs by value (`memcpy`).
* **Automated Visual QA**:
  * A host-native (`x86_64` / `aarch64`) harness compiles offscreen to render pixel-perfect snapshots to PNG with synthetic touch injections, providing automated layout verification in CI/CD before flashing real boards.
