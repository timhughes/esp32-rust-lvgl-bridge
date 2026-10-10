# System Architecture

## Dual-Core Task Model

When running on dual-core Espressif SoCs (ESP32-S3 and ESP32-P4), the application is partitioned cleanly across FreeRTOS cores:

```text
┌─────────────────────────────────┐       ┌─────────────────────────────────┐
│     Core 0: Rust State Core     │       │     Core 1: C / LVGL GUI Core   │
│                                 │       │                                 │
│  ┌───────────────────────────┐  │       │  ┌───────────────────────────┐  │
│  │ Rust Control Worker Task  │  │       │  │ LVGL UI / Rendering Task  │  │
│  │  - Closed-loop control    │  │       │  │  - LVGL v9 Scene Graph    │  │
│  │  - 20ms fixed step rate   │  │       │  │  - ST7701S RGB DMA Scanout│  │
│  │  - State machines & logic │  │       │  │  - GT911 Touch Polling    │  │
│  └─────────────┬─────────────┘  │       │  └─────────────▲─────────────┘  │
└────────────────┼────────────────┘       └────────────────┼────────────────┘
                 │                                         │
                 │ FreeRTOS Queue (Value Copy)             │ FreeRTOS Queue (Value Copy)
                 ▼                                         ▼
         ┌─────────────────────────────────────────────────────────┐
         │              FFI Bridge Protocol (POD Types)            │
         │  - UiCommand (Control inputs from user touch)           │
         │  - DeviceTelemetry (State broadcasts from Rust worker)  │
         └─────────────────────────────────────────────────────────┘
```

### Core 0: Rust Worker (`rust_worker_task`)
* Priority: 5, Stack: 8 KB
* Periodically steps the deterministic domain logic (20ms step rate / 50 Hz).
* Consumes commands sent from the UI via `bridge_recv_command`.
* Broadcasts updated telemetry metrics to the UI via `bridge_send_telemetry`.

### Core 1: LVGL UI Task (`lvgl_ui_task`)
* Priority: 5, Stack: 8 KB
* Drives LVGL v9 display rendering and touch input reading.
* Invokes `pump_incoming_telemetry()` on each loop tick to apply pending updates to data-bound widgets.
* Handles touch events, debouncing, and holdoff leases to prevent UI stutter while dragging controls.
