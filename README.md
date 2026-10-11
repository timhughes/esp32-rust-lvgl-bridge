# Heterogeneous ESP32 (P4, C6, S3) Hybrid C/Rust Architecture

[![Architecture: C + Rust](https://img.shields.io/badge/Architecture-C%20%2B%20Rust%20(no__std)-blue.svg)](#core-architectural-principle)
[![LVGL: v9](https://img.shields.io/badge/LVGL-v9.4-orange.svg)](https://lvgl.io)
[![ESP-IDF: v5.4+](https://img.shields.io/badge/ESP--IDF-v5.4%2B-red.svg)](https://github.com/espressif/esp-idf)
[![Target: P4 | S3 | C6](https://img.shields.io/badge/Targets-ESP32--P4%20|%20S3%20|%20C6-green.svg)](#target-hardware-matrix)

A production-grade, multi-target embedded firmware architecture and automated headless visual QA pipeline for modern Espressif microcontrollers.

---

## 1. Core Architectural Principle

> **"C owns the display; Rust owns the system."**

* **C / ESP-IDF:** Drives display controllers, DMA framebuffers, touch digitizers, and the LVGL v9 scene graph.
* **Rust (`no_std` `staticlib`):** Implements all business logic, closed-loop state machines, protocol decoding, and hardware workers.
* **FFI Boundary:** Zero shared heap pointers, zero `lv_obj_t*` leaking into Rust. Communication occurs exclusively across typed FreeRTOS queues passing fixed-size Plain Old Data (POD) structs by value via `memcpy`.
* **Headless Visual QA:** A host-native (`x86_64` / `aarch64`) harness renders frames offscreen to RGB888 PNG with mock touch inputs, allowing automated visual auditing of layouts, fonts, and state machines in CI/CD without physical hardware.

---

## 2. Target Hardware Matrix & Roles

| Target | Architecture | Core Freq | Primary Peripherals | Hardware Role |
|---|---|---|---|---|
| **ESP32-P4** | Dual RISC-V (HP) | 400 MHz | MIPI-DSI, Hardware PPA, USB 2.0 HS (480M), Eth MAC | **Main GUI Application Processor** |
| **ESP32-S3** | Dual Xtensa LX7 | 240 MHz | Vector AI instructions, Parallel RGB LCD, USB-OTG | **Display & Edge ML Node** |
| **ESP32-C6** | Single RISC-V (LP) | 160 MHz | Wi-Fi 6 (2.4 GHz), Bluetooth 5.3, 802.15.4 (Thread/Zigbee/Matter) | **Low-Power Wireless Co-processor** |

---

## 3. Repository Layout

```text
esp32-rust-lvgl-framework/
├── CMakeLists.txt                       # Top-level ESP-IDF CMake orchestration
├── .cargo/
│   └── config.toml                      # Target runners & linker configurations
├── Cargo.toml                           # Cargo workspace definition
├── crates/
│   ├── bridge-core/                     # Shared no_std ABI contracts & worker traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                   # Re-exports ABI and worker traits
│   │       ├── abi.rs                   # ParamId, UiCommand, DeviceTelemetry POD structs
│   │       └── worker.rs                # Generic Controller & WorkerRunner traits
│   └── app-core/                        # Application domain logic (compiled to staticlib)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs                   # SystemEngine state machine & C FFI exports
├── main/                                # ESP-IDF firmware component
│   ├── CMakeLists.txt                   # Cargo compilation bridge & C target link
│   ├── idf_component.yml                # Managed dependencies (LVGL v9, drivers)
│   ├── include/
│   │   ├── bridge_protocol.h            # C FFI declarations matching Rust ABI
│   │   ├── app_ui.h                     # UI hierarchy declarations & context accessors
│   │   ├── ui_binding.h                 # Declarative single-line widget binding engine
│   │   └── p4_c6_link.h                 # P4 <-> C6 Inter-chip link protocol
│   ├── bridge_transport.c               # FreeRTOS queue wrappers for Rust
│   ├── ui_binding.c                     # Declarative UI arbitration & binding implementation
│   ├── p4_c6_link.c                     # SDIO 3.0 / High-Speed SPI driver with CRC-CCITT
│   └── main.c                           # Dual-core task setup & UI declaration
└── test_harness/                        # Host-native headless test pipeline
    ├── Makefile                         # Native host GCC/Clang build
    ├── lv_conf.h                        # Host LVGL configuration (RGB888, CLIB malloc)
    ├── stb_image_write.h                # Single-header PNG exporter
    ├── host_transport.h                 # Synchronous in-memory FIFO header
    ├── host_transport.c                 # Replaces FreeRTOS queues for deterministic testing
    ├── harness.h                        # Offscreen display & mock touch injector API
    ├── harness.c                        # RGB888 canvas implementation & PNG writer
    ├── main_test.c                      # Multi-scenario automated test runner
    └── artifacts/                       # Generated offscreen test snapshots
        ├── 01_initial_state.png
        ├── 02_switch_toggled.png
        ├── 03_slider_dragging.png
        ├── 04_slider_released.png
        ├── 05_slider_telemetry_settled.png
        ├── 06_counter_reset.png
        ├── 07_tier1_showcase.png
        ├── 08_tier2_showcase.png
        ├── 09_tier3_telemetry.png
        ├── 10_tier3_diagnostics.png
        └── 11_tier3_config.png
```

---

## 4. Communication Protocol & FFI Contract

Communication between C (LVGL) and Rust (Domain Logic) is strictly constrained to **Plain Old Data (POD)** structs passed by value across FreeRTOS queues (zero shared heap, zero raw pointers). Full specification: [`docs/src/ffi_contracts.md`](file:///home/thughes/git/esp32-rust-lvgl-framework/docs/src/ffi_contracts.md).

```c
typedef enum {
    PARAM_SETPOINT_HZ    = 1,  // Slider: i32_val setpoint
    PARAM_SYSTEM_ENABLE  = 2,  // Switch: bool_val enable/run state
    PARAM_TRIGGER_RESET  = 3,  // Push Button: momentary command trigger
    PARAM_CYCLE_COUNT    = 4,  // Readout: u32_val telemetry counter
    // ... Supports 25 parameters across Tier 1, Tier 2, and Tier 3 widgets
} param_id_t;

typedef union {
    int32_t  i32_val;
    uint32_t u32_val;
    float    f32_val;
    bool     bool_val;
    char     str_val[32];      // Zero-allocation fixed string buffer
} param_val_t;

// UI -> Rust Worker (Command Queue)
typedef struct {
    param_id_t  param_id;
    uint32_t    seq_id;
    param_val_t value;
} ui_command_t;

// Rust Worker -> UI (Telemetry Queue)
typedef struct {
    param_id_t  param_id;
    uint32_t    ack_seq_id;
    param_val_t value;
} device_telemetry_t;
```

---

## 5. UI Arbitration & Synchronization Strategy

To prevent race conditions, visual stutter, and telemetry echo while a user interacts with widgets, the framework utilizes a **3-stage arbitration engine**:

```
[User Interacting] ────────► UI Leased by User (Incoming Telemetry Dropped)
                                      │
[Finger Released]  ────────► Holdoff Timer Starts (USER_HOLDOFF_MS = 500ms)
                                      │
                             Worker Acknowledges Sequence ID
                                      │
[Holdoff Expires & Acked] ──► UI Returns to Worker Telemetry Tracking
```

### Core Invariants
1. **Monotonic Sequence IDs (`seq_id`):** Every user interaction increments a global sequence counter. Commands transmit this ticket. Rust latches this ID and mirrors it as `ack_seq_id`. Packets from older generations (`ack_seq_id < current_seq`) are discarded on receipt.
2. **Temporal Holdoff Lease (`USER_HOLDOFF_MS = 500`):** Prevents physical systems with settling times (e.g. motor ramp-up or closed-loop filter lag) from snapping the UI slider back to an intermediate ramping value immediately upon release.
3. **Active Touch Priority:** If `lv_obj_has_state(widget, LV_STATE_PRESSED)` is true, external telemetry updates are dropped immediately to eliminate visual rubber-banding.
4. **Programmatic Update Guards:** When syncing widgets (like switches) from worker telemetry, an internal `is_programmatic_update = true` flag suppresses outbound `LV_EVENT_VALUE_CHANGED` callbacks recursively.

### Declarative Data-Driven UI Schema (Authoring Widgets 100% in Rust)

The framework supports **100% Rust-defined UI schemas**. Developers can declare widgets, layouts, parameters, and labels directly in `crates/app-core/src/lib.rs` without modifying any C display code:

```rust
// Declare the interface in Rust (crates/app-core/src/lib.rs)
pub static UI_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        20, 20, // X, Y offset
        c"System: RUNNING",
        c"System: STOPPED",
    ),
    WidgetDescriptor::slider(
        ParamId::SetpointHz,
        UiAlign::Center,
        0, 15, // Centered with +15px Y offset
        1, 100, // Min, Max range
        10,     // Initial value
        500,    // Holdoff lease ms
        c"%ld Hz (Setting)",
        c"%ld Hz (Actual)",
    ),
    WidgetDescriptor::button(
        ParamId::TriggerReset,
        UiAlign::BottomLeft,
        20, -20,
        c"Reset Counter",
        1,      // Command value
    ),
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomLeft,
        170, -30,
        c"Cycles: %lu",
    ),
];
```

The C side (`main/main.c`) acts as a **pure rendering engine**, automatically creating and binding the entire LVGL scene graph at startup in a single call:

```c
void setup_test_ui(void) {
    ui_engine_init();
    size_t count = 0;
    const widget_descriptor_t *schema = rust_get_ui_schema(&count);
    ui_engine_build_from_schema(schema, count);
}
```

### Supported LVGL v9 Widget Catalog (25 Widgets)
The declarative engine supports 25 LVGL v9 widgets out of the box without requiring custom C UI assembly:
* **Tier 1 (Scalars & Selectors)**: `Switch`, `Slider`, `Button`, `Label`, `Arc` (Knob/Gauge), `Bar`, `Checkbox`, `LED`, `Spinner`, `Dropdown`, `Roller`, `Spinbox`.
* **Tier 2 (Text, Keypads & Modals)**: `TextArea`, `Keyboard` (auto-focus binding), `ButtonMatrix` (PIN pad/keypad), `MsgBox` (alert overlays), `Scale` (analog dial meters).
* **Tier 3 (Containers & Data Streams)**: `TabView` (multi-screen navigation with tab routing), `TileView`, `Menu`, `List`, `Chart` (streaming waveform oscilloscope), `Table` (system diagnostic matrix), `Calendar` (date picker), `Canvas` (custom graphics).

See [`docs/src/declarative_ui.md`](file:///home/thughes/git/esp32-rust-lvgl-framework/docs/src/declarative_ui.md) for full constructor APIs, schema definitions, and container routing rules.

---

## 6. P4 <-> C6 Inter-Chip Communication Link

On the **ESP32-P4**, high-speed wireless connectivity is delegated to an auxiliary **ESP32-C6** co-processor over **SDIO 3.0** or **High-Speed SPI** (`main/include/p4_c6_link.h`):

* **Channel Multiplexing:**
  * `P4_C6_CHAN_SYSTEM`: Reset, power state, clock sync, heartbeats
  * `P4_C6_CHAN_WIFI`: 802.3 Ethernet frame encapsulation (ESP-Hosted)
  * `P4_C6_CHAN_BLE`: HCI commands, events, and data packets (Bluetooth 5.3)
  * `P4_C6_CHAN_802154`: Thread, Zigbee, and Matter frames
  * `P4_C6_CHAN_TELEMETRY`: Co-processor RF metrics and health
* **Framing & Integrity:** Every packet contains a 16-bit sync magic (`0xE5P4`), sequence ticket, payload length, and CRC-CCITT checksum.
* **Host Simulation:** Includes a built-in zero-copy loopback mock mode for host-native unit testing.

---

## 7. Headless Host Test Suite (Visual QA in CI/CD)

The test suite runs natively on Linux/macOS without requiring an attached ESP32 development board.

### Running the Suite
```bash
cd test_harness
make run
```

### Generated Artifacts (`test_harness/artifacts/`)
* **`01_initial_state.png`**: System stopped, slider at default 10 Hz, cycle counter at 0.
* **`02_switch_toggled.png`**: Switch toggled to RUNNING, background worker begins stepping (`Cycles: 10`).
* **`03_slider_dragging.png`**: Touch injected dragging slider to 60 Hz; label reads `60 Hz (Setting)`.
* **`04_slider_released.png`**: Touch lifted; 500ms holdoff lease actively suppresses intermediate ramping telemetry.
* **`05_slider_telemetry_settled.png`**: Holdoff expired; label confirms `60 Hz (Actual)`, cycles incrementing.
* **`06_counter_reset.png`**: Push button clicked; cycle count atomically clears to 0 and resumes.
* **`07_tier1_showcase.png`**: 480×480 screen with all Tier 1 widgets active (Arc knob, Switch, LED, Bar, Spinner, Checkbox, Dropdown, Roller, Spinbox).
* **`08_tier2_showcase.png`**: 480×480 screen with Tier 2 widgets active (TextArea pre-filled, On-Screen Keyboard, Numeric/Keypad ButtonMatrix, Analog Speedometer Scale, MsgBox alert overlay).
* **`09_tier3_telemetry.png`**: Tier 3 TabView with Tab 0 ("Telemetry") active: real-time streaming waveform Chart, interactive Arc knob, and system status Bar.
* **`10_tier3_diagnostics.png`**: Tier 3 TabView with Tab 1 ("Diagnostics") active: multi-row system status Table (`Module`, `Status`, `Core 0`, `Core 1`, `PSRAM`) and scrollable Item List.
* **`11_tier3_config.png`**: Tier 3 TabView with Tab 2 ("Config") active: interactive Calendar date picker and raw-pixel graphics Canvas.

---

## 8. Embedded Firmware Build (ESP-IDF)

### General Prerequisites
* ESP-IDF **v5.4+** or **v5.5+**
* Python 3.10+ with `esptool` installed
* Rust toolchain with target support:
  * **ESP32-P4**: `rustup target add riscv32imafc-unknown-none-elf`
  * **ESP32-C6**: `rustup target add riscv32imac-unknown-none-elf`
  * **ESP32-S3**: Espressif LLVM Xtensa toolchain (`rust-src` + `xtensa-esp32s3-none-elf`)

---

### ESP32-S3-4848S040 (Guition 4.0" 480×480 Smart Display)

The **ESP32-S3-4848S040** board features an ESP32-S3-WROOM-1 module with:
* **Flash:** 16MB Quad-SPI Flash (80MHz, DIO mode)
* **PSRAM:** 8MB Octal PSRAM (80MHz, OPI mode)
* **Display:** 4.0-inch 480×480 IPS LCD driven by an ST7701S controller via 16-bit parallel RGB bus + 3-wire 9-bit SPI configuration interface
* **Touch Digitizer:** Goodix GT911 capacitive touch over I2C (SDA: GPIO 19, SCL: GPIO 45)
* **Backlight:** GPIO 38

#### Critical Platform Considerations
1. **Xtensa Windowed ABI (`.cargo/config.toml`)**:
   Rust entry points and FFI bridges (`rust_worker_init`, `rust_worker_step`, `bridge_recv_command`, `bridge_send_telemetry`) are called directly by FreeRTOS C tasks. Xtensa Windowed ABI must be explicitly enabled in `.cargo/config.toml` to prevent IllegalInstruction / WindowUnderflow panics:
   ```toml
   [target.xtensa-esp32s3-none-elf]
   rustflags = ["-C", "target-feature=+exception,+windowed"]
   ```
2. **ST7701S MDT Register (0xCD) Fix**:
   The ST7701S controller on Guition 4848S040 panels requires Command 2 Bank 0 register `0xCD` to be set to `0x00`. Without this command, the panel defaults to 18-bit sub-pixel mode, scrambling 16-bit RGB565 parallel data into vertical stripes across the screen. The custom sequence is implemented in `main/bsp_4848s040.c`.

#### Building for ESP32-S3
```bash
# 1. Source ESP-IDF environment
. $HOME/esp/esp-idf/export.sh

# 2. Build the firmware (compiles Rust crates and C firmware)
idf.py -DIDF_TARGET=esp32s3 build
```

#### Flashing to Physical Hardware
Connect the board via the USB-C serial port (typically `/dev/ttyUSB0` on Linux):
```bash
# Flash bootloader, partition table, and application at 460800 baud:
cd build
python -m esptool --chip esp32s3 -p /dev/ttyUSB0 -b 460800 \
    --before default_reset --after hard_reset \
    write_flash "@flash_args"
```

#### Serial Monitoring
```bash
# Monitor dual-core logs at 115200 baud (Ctrl+] to exit):
idf.py -p /dev/ttyUSB0 monitor
```

---

## 9. Rust Package Documentation System

In Python, packages typically use Sphinx or MkDocs. In the Rust ecosystem, documentation is handled by two standard official tools:

### 1. `rustdoc` & `cargo doc` (API Reference & Docstrings)
`rustdoc` is the built-in, compiler-native documentation tool for Rust crates:
* **Doc Comments**: Use `///` for documenting public items (structs, functions, enums) and `//!` for crate/module level documentation. All doc comments support GitHub-Flavored Markdown.
* **Doctests**: Any code snippets written in `/// ``` ` blocks are compiled and executed during `cargo test`. If an example in the docs becomes outdated or fails to compile, CI immediately catches it.
* **Building Docs Locally**:
  ```bash
  # Generate and view HTML documentation for all crates in the workspace:
  cargo doc --workspace --no-deps --open
  ```
  This creates a searchable, hyperlinked static HTML site in `target/doc/`.
* **Central Hosting**: Any open-source crate published to crates.io is automatically built and hosted on [docs.rs](https://docs.rs).

### 2. `mdBook` (User Guides, Manuals, & Architecture Books)
While `cargo doc` produces API reference documentation, **`mdBook`** is the standard tool in Rust for writing multi-page user guides and tutorials (analogous to Sphinx / ReadTheDocs):
* Maintained officially by the Rust project and used for *The Rust Programming Language*, *The Embedded Rust Book*, and *The Cargo Book*.
* Simple structure: Markdown files organized via a `SUMMARY.md` table of contents.
* Install and run:
  ```bash
  cargo install mdbook
  mdbook init docs
  mdbook serve docs --open
  ```

---

## 10. License & Contributing
* Firmware C/Rust codebase: Licensed under Apache 2.0 / MIT.
* LVGL is licensed under MIT.
* `stb_image_write.h` is Public Domain / MIT.
