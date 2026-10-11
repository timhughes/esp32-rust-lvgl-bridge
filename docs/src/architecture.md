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
* Invokes `ui_engine_pump_telemetry()` on each loop tick to apply pending updates to data-bound widgets.
* Handles touch events, debouncing, and holdoff leases to prevent UI stutter while dragging controls.

---

## Modular Component & Library Architecture

The framework is organized as reusable, decoupled libraries suitable for direct integration into third-party projects:

```text
components/
├── esp_lvgl_bridge/          # Core ESP-IDF component (C engine)
│   ├── include/
│   │   ├── esp_lvgl_bridge.h # Umbrella public header
│   │   ├── bridge_protocol.h # FFI contracts & POD structs
│   │   ├── ui_binding.h      # Declarative schema parser & arbitration engine
│   │   ├── esp_lvgl_bsp.h    # Generic hardware BSP interface
│   │   └── p4_c6_link.h      # High-speed co-processor link
│   ├── cmake/
│   │   └── esp_rust.cmake    # Reusable CMake helper for building & linking Rust
│   └── Kconfig               # Menuconfig settings (queue depths, holdoff, timeouts)
└── bsp_guition_s3_4848s040/  # Pluggable Board Support Package component
    ├── include/bsp_4848s040.h # Display, touch, and backlight HAL interface
    └── src/bsp_4848s040.c     # ST7701S RGB parallel bus & GT911 touch driver

crates/
├── bridge-core/              # no_std Rust crate: ABI contracts & open ParamId newtype
├── app-core/                 # Application staticlib: state machine & declarative schemas
└── minimal-app/              # Minimal starter crate (examples/minimal)
```

### Generic Hardware BSP Interface (`esp_lvgl_bsp_ops_t`)

To completely decouple the bridge and application from board-specific hardware drivers, display panel buses, and touch digitizers, the framework defines a standard operations table in [`esp_lvgl_bsp.h`](file:///home/thughes/git/esp32-rust-lvgl-framework/components/esp_lvgl_bridge/include/esp_lvgl_bsp.h):

```c
typedef struct {
    const char   *board_name;
    lv_display_t *(*init_display)(void);
    lv_indev_t   *(*init_touch)(lv_display_t *disp);
    void          (*set_backlight)(uint8_t percent);
    bool          (*backlight_is_on)(void);
} esp_lvgl_bsp_ops_t;
```

Any Board Support Package component (such as [`bsp_guition_s3_4848s040`](file:///home/thughes/git/esp32-rust-lvgl-framework/components/bsp_guition_s3_4848s040)) exports an instance of this struct. The application initializes hardware without coupling to driver internals:

```c
const esp_lvgl_bsp_ops_t *bsp = bsp_4848s040_get_ops();
ESP_LOGI("APP", "Initializing BSP: %s...", bsp->board_name);
lv_display_t *disp = bsp->init_display();
bsp->init_touch(disp);

// Backlight and screensaver power control:
bsp->set_backlight(0);   // Blank display after 60s inactivity
bsp->set_backlight(100); // Wake on touch
```

### CMake Integration with `esp_rust.cmake`
Any user project can link a Rust crate to its ESP-IDF component using the framework's helper:
```cmake
include(${ESP_LVGL_BRIDGE_DIR}/cmake/esp_rust.cmake)
esp_rust_link_staticlib(
    COMPONENT ${COMPONENT_LIB}
    CRATE_NAME "my-rust-crate"
    CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/rust"
)
```
