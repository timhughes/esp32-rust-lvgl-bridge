# ESP32 Hybrid C/Rust LVGL Framework

[![Architecture: C + Rust](https://img.shields.io/badge/Architecture-C%20%2B%20Rust%20(no__std)-blue.svg)](#1-overview--architectural-philosophy)
[![LVGL: v9](https://img.shields.io/badge/LVGL-v9.4-orange.svg)](https://lvgl.io)
[![ESP-IDF: v5.4+](https://img.shields.io/badge/ESP--IDF-v5.4%2B-red.svg)](https://github.com/espressif/esp-idf)
[![Target: S3 | P4 | C6](https://img.shields.io/badge/Targets-ESP32--S3%20|%20P4%20|%20C6-green.svg)](#3-supported-hardware-matrix)

A production-ready embedded firmware framework and automated visual QA testing pipeline for modern Espressif microcontrollers (**ESP32-S3**, **ESP32-P4**, **ESP32-C6**).

This project combines the industry-standard **LVGL v9** graphics engine in C with safe, deterministic **`no_std` Rust** for domain logic, closed-loop state machines, and protocols.

---

## Table of Contents

1. [Overview & Architectural Philosophy](#1-overview--architectural-philosophy)
2. [Key Features](#2-key-features)
3. [Supported Hardware Matrix](#3-supported-hardware-matrix)
4. [Prerequisites & Dependencies](#4-prerequisites--dependencies)
5. [Quickstart Guide](#5-quickstart-guide)
   - [Option A: Run Headless Simulation on Host (No Hardware Needed)](#option-a-run-headless-simulation-on-host-no-hardware-needed)
   - [Option B: Build & Flash to Physical Hardware](#option-b-build--flash-to-physical-hardware)
6. [How It Works](#6-how-it-works)
   - [FFI Communication Protocol (Zero-Copy POD Structs)](#ffi-communication-protocol-zero-copy-pod-structs)
   - [3-Stage Arbitration Engine (Jitter-Free Controls)](#3-stage-arbitration-engine-jitter-free-controls)
   - [Declarative UI Defined 100% in Rust](#declarative-ui-defined-100-in-rust)
   - [Full 26-Widget Catalog](#full-26-widget-catalog)
7. [Developer Tutorial: Adding Your Own Feature](#7-developer-tutorial-adding-your-own-feature)
8. [Hardware Deep Dive: Guition ESP32-S3-4848S040](#8-hardware-deep-dive-guition-esp32-s3-4848s040)
   - [Display & Pinout Specifications](#display--pinout-specifications)
   - [Automatic 1-Minute Screen Timeout & Wake-on-Touch](#automatic-1-minute-screen-timeout--wake-on-touch)
   - [ST7701S RGB MDT (0xCD) Parallel Bus Fix](#st7701s-rgb-mdt-0xcd-parallel-bus-fix)
9. [Inter-Chip Link (ESP32-P4 + ESP32-C6)](#9-inter-chip-link-esp32-p4--esp32-c6)
10. [Automated Visual QA & Testing](#10-automated-visual-qa--testing)
11. [Repository Structure](#11-repository-structure)
12. [License](#12-license)

---

## 1. Overview & Architectural Philosophy

### *"C owns the display; Rust owns the system."*

Building complex graphical embedded applications typically forces a difficult choice:
- **Pure C/C++:** Fast access to vendor drivers and LVGL, but prone to memory safety bugs, race conditions, and unstructured state machines.
- **Pure Rust:** Memory-safe and expressive, but wrapping massive C GUI libraries like LVGL requires thousands of lines of fragile `unsafe` FFI bindings, complex pointer lifetimes, and heavy build workarounds.

This framework solves the problem by cleanly separating concerns across a strict hardware/domain boundary:

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

1. **C / ESP-IDF (Display & Hardware Layer):** Drives display controllers, DMA framebuffers, touch digitizers, FreeRTOS tasks, and the LVGL v9 scene graph.
2. **Rust `no_std` (Application Core):** Implements all business logic, closed-loop state machines, protocols, and data models inside an isolated `staticlib`.
3. **POD FFI Boundary:** Communication occurs strictly via value-typed FreeRTOS ring queues passing fixed-size Plain Old Data (POD) structs. Zero shared heap memory, zero raw `lv_obj_t*` pointers crossing into Rust.

---

## 2. Key Features

- **100% Declarative UI Schemas in Rust:** Declare widgets, layouts, parameters, and labels directly in Rust static tables. The C engine constructs and binds the scene graph dynamically with a single call.
- **Complete 26-Widget LVGL Suite:** Out-of-the-box support for all 26 core LVGL widgets across 3 tiers (switches, sliders, dials, gauges, keyboards, text inputs, charts, tables, calendar, canvas, menus, tileviews).
- **Dynamic Virtual Keyboard Focus:** Tap a single-line command field or multi-line text area, and the on-screen QWERTY keyboard automatically retargets its input focus.
- **Intelligent Inactivity Timeout & Wake-on-Touch:** Turns off the display backlight after 60 seconds of idle time to conserve power and prolong panel life. Tapping the dark screen immediately restores full brightness without triggering accidental clicks.
- **Anti-Jitter 3-Stage Arbitration:** Eliminates visual rubber-banding and telemetry echo during user interaction via monotonic sequence tickets (`seq_id`) and temporal holdoff leases.
- **Headless Host Visual QA (CI/CD):** Render pixel-perfect offscreen screenshots of the entire UI and verify state transitions on Linux/macOS in milliseconds without hardware attached.
- **Turnkey Guition ESP32-S3-4848S040 Support:** Full board-support package for 4.0-inch 480×480 IPS display with ST7701S parallel RGB bus and Goodix GT911 touch.

---

## 3. Supported Hardware Matrix

| Hardware Target | Microcontroller Architecture | Display & Interfaces | Primary Role |
|---|---|---|---|
| **Guition 4848S040** | ESP32-S3 Dual Xtensa LX7 (240MHz, 8MB PSRAM, 16MB Flash) | 4.0" 480×480 ST7701S 16-bit RGB Parallel, GT911 I2C Touch | **Turnkey Smart Display Device** |
| **ESP32-P4** | Dual RISC-V HP (400MHz, MIPI-DSI, HW PPA 2D Accelerator) | High-definition MIPI-DSI Displays, USB 2.0 High-Speed | **High-Performance GUI Host** |
| **ESP32-C6** | Single RISC-V LP (160MHz, Wi-Fi 6, BLE 5.3, 802.15.4) | SDIO 3.0 / High-Speed SPI link to P4 | **Wireless Co-Processor Node** |
| **Host PC (x86_64 / aarch64)** | Native Linux / macOS | Headless offscreen framebuffer (generates RGB888 PNGs) | **Visual QA & Automated Testing** |

---

## 4. Prerequisites & Dependencies

### For Host Simulation & Testing (No Hardware Required)
- **OS:** Linux (Ubuntu 22.04+, Debian 12+, Fedora 38+, Arch) or macOS.
- **Build Tools:** `gcc` or `clang`, `make`.
- **Rust:** Standard Rust toolchain (`rustup install stable` or `nightly`).
- **Graphics Libraries:** None! The host harness includes an offscreen software renderer and single-header PNG exporter (`stb_image_write.h`).

### For Flashing Physical Hardware (ESP32-S3)
- **ESP-IDF:** v5.4+ or v5.5+ ([Installation Guide](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/get-started/)).
- **Python:** 3.10+ with `esptool` installed.
- **Rust Toolchain:**
  - Rust nightly toolchain with `rust-src` component:
    ```bash
    rustup toolchain install nightly
    rustup component add rust-src --toolchain nightly
    ```
  - Espressif Xtensa LLVM target (configured automatically via `.cargo/config.toml`).

---

## 5. Quickstart Guide

### Option A: Run Headless Simulation on Host (No Hardware Needed)

You can compile and run the full LVGL v9 engine, Rust state machine, and test harness on your PC in under 10 seconds:

```bash
# 1. Clone the repository
git clone https://github.com/your-org/esp32-rust-lvgl-framework.git
cd esp32-rust-lvgl-framework

# 2. Build the Rust domain logic staticlib
cargo build --release -p app-core

# 3. Build and execute the visual test harness
cd test_harness
make run
```

The test runner executes 4 headless unit test suites (FFI serialization, P4-C6 link, Rust state engine, 3-stage arbitration) followed by 12 interactive visual scenarios (including 6-tab S3 app and screensaver power management):

```text
--- Phase 1: Unit Assertions (Headless Protocol & Engine) ---
  [TEST] FFI Protocol & POD Serialization... -> PASS
  [TEST] Inter-Chip Link (P4 <-> C6) Loopback & Protocol Framing... -> PASS
  [TEST] Rust SystemEngine Closed-Loop State Simulation... -> PASS
  [TEST] 3-Stage Anti-Jitter Arbitration Engine... -> PASS

--- Phase 2: Flagship S3-4848 6-Tab Application Journey ---
[1/12] S3 Tab 0 (Controls) Initial State -> Saved artifacts/01_s3_tab0_controls_init.png
[2/12] S3 Tab 0 (Controls) Interactive Running & Dragging -> Saved artifacts/02_s3_tab0_controls_running.png
...
[7/12] S3 Tab 5 (System) Settings Menu & Swipeable TileView -> Saved artifacts/07_s3_tab5_system.png

--- Phase 3: Power Management & Inactivity Lifecycle ---
[8/12] Inactivity Timeout (Screen Blanked after 60s) -> Saved artifacts/08_s3_screen_blanked.png
[9/12] Wake-on-Touch (Screen Restored, Blind Tap Swallowed) -> Saved artifacts/09_s3_screen_woken.png

--- Phase 4: Standalone Widget Catalog Smoke Tests ---
[10/12] Tier 1 12-Widget Showcase -> Saved artifacts/10_tier1_showcase.png
[11/12] Tier 2 Text, Keyboard, Matrix & Dialog Showcase -> Saved artifacts/11_tier2_showcase.png
[12/12] Tier 3 Multi-Tab Container Showcase -> Saved artifacts/12_tier3_showcase.png
```

> [!TIP]
> Open any generated `.png` in your image viewer to inspect the exact pixels rendered by LVGL.

---

### Option B: Build & Flash to Physical Hardware

Follow these steps to deploy the full 26-widget demo application to a **Guition ESP32-S3-4848S040** display:

```bash
# 1. Connect your ESP32-S3 board to USB (e.g. /dev/ttyUSB0 on Linux)

# 2. Source the ESP-IDF environment
source $HOME/esp/esp-idf/export.sh

# 3. Build the Rust static library for Xtensa bare-metal
cargo build --release --target xtensa-esp32s3-none-elf -p app-core

# 4. Build the ESP-IDF firmware binary
idf.py -DIDF_TARGET=esp32s3 build

# 5. Flash to the connected board at 460800 baud
idf.py -p /dev/ttyUSB0 -b 460800 flash

# 6. Monitor live serial logs (Core 0 Rust engine + Core 1 LVGL UI)
idf.py -p /dev/ttyUSB0 monitor
```

*(Press `Ctrl+]` to exit the serial monitor).*

---

## 6. How It Works

### FFI Communication Protocol (Zero-Copy POD Structs)

All communication between C (LVGL) and Rust (`SystemEngine`) uses Plain Old Data (POD) structs passed by value over FreeRTOS queues ([`main/include/bridge_protocol.h`](file:///home/thughes/git/esp32-rust-lvgl-framework/main/include/bridge_protocol.h) and [`crates/bridge-core/src/abi.rs`](file:///home/thughes/git/esp32-rust-lvgl-framework/crates/bridge-core/src/abi.rs)):

```rust
// Fixed-size Plain Old Data parameter value (no heap pointers!)
#[repr(C)]
#[derive(Copy, Clone)]
pub union ParamVal {
    pub i32_val: i32,
    pub u32_val: u32,
    pub f32_val: f32,
    pub bool_val: bool,
    pub str_val: [u8; 32], // Zero-allocation fixed string buffer
}

// UI -> Rust Worker (Command Queue)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UiCommand {
    pub param_id: ParamId,
    pub seq_id: u32,
    pub value: ParamVal,
}

// Rust Worker -> UI (Telemetry Queue)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DeviceTelemetry {
    pub param_id: ParamId,
    pub ack_seq_id: u32,
    pub value: ParamVal,
}
```

---

### 3-Stage Arbitration Engine (Jitter-Free Controls)

When a user drags a slider or turns a rotary dial, high-rate background telemetry updates can cause "rubber-banding" if the motor or filter has a physical lag. The framework resolves this with a **3-stage arbitration lease**:

```
[User Touching]   ──► ARB_DRAGGING: UI leased by user; incoming telemetry dropped.
                           │
[User Released]   ──► ARB_HOLDOFF: 500ms lease active; worker acknowledges seq_id.
                           │
[Holdoff Expired] ──► ARB_TRACKING: UI smoothly returns to worker telemetry tracking.
```

- **Monotonic Sequence IDs (`seq_id`):** Every touch interaction claims a monotonically increasing sequence ticket. Stale telemetry packets with `ack_seq_id < current_seq` are discarded.
- **Active Touch Priority:** If `lv_obj_has_state(widget, LV_STATE_PRESSED)` is true, external telemetry is ignored immediately.
- **Recursion Suppression:** When updating toggles programmatically from telemetry, `is_programmatic_update = true` suppresses outbound echo loops.

---

### Declarative UI Defined 100% in Rust

Instead of writing hundreds of lines of procedural C layout code, the interface is defined as static data tables in Rust ([`crates/app-core/src/lib.rs`](file:///home/thughes/git/esp32-rust-lvgl-framework/crates/app-core/src/lib.rs)):

```rust
pub static S3_4848_SCHEMA: &[WidgetDescriptor] = &[
    // 1. Root Container: 6-Tab TabView across 480x480 screen
    WidgetDescriptor::tabview(
        ParamId::NavTab,
        c"Controls\nSelectors\nInputs\nTelemetry\nTools\nSystem",
        480, 480,
    ),

    // 2. Power Toggle Switch on Tab 0
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        20, 15,
        c"System: RUNNING",
        c"System: STOPPED",
    ).with_parent(ParamId::NavTab, 0),

    // 3. Frequency Slider with 500ms Holdoff Arbitration on Tab 0
    WidgetDescriptor::slider(
        ParamId::SetpointHz,
        UiAlign::Center,
        0, 95,
        1, 100, 10, 500,
        c"%ld Hz (Setting)",
        c"%ld Hz (Actual)",
    ).with_parent(ParamId::NavTab, 0),

    // 4. Single-line Text Input on Tab 2
    WidgetDescriptor::text_input(
        ParamId::TextInput,
        UiAlign::TopMid,
        0, 10, 440, 40,
        c"Single-line command...",
        Some(c"SYS:STATUS"),
        31,
    ).with_parent(ParamId::NavTab, 2),

    // 5. Virtual On-Screen Keyboard on Tab 2
    WidgetDescriptor::keyboard(
        ParamId::Keyboard,
        ParamId::TextInput,
        UiAlign::BottomMid,
        0, -5, 460, 165,
    ).with_parent(ParamId::NavTab, 2),
];
```

The C runtime binds this table into native LVGL objects at boot with a single call:
```c
ui_engine_build_from_schema(schema, count);
```

---

### Full 26-Widget Catalog

The framework includes complete implementations for all 26 widget types:

| Category | Widgets |
|---|---|
| **Tier 1: Scalars & Selectors** | `Switch`, `Slider`, `Button`, `Label`, `Arc` (Rotary Knob), `Bar`, `Checkbox`, `LED`, `Spinner`, `Dropdown`, `Roller`, `Spinbox` |
| **Tier 2: Text, Keys & Gauges** | `TextInput` (single-line), `TextArea` (multi-line), `Keyboard` (virtual QWERTY), `ButtonMatrix` (PIN pad), `Scale` (analog needle meter), `MsgBox` (modal alert) |
| **Tier 3: Containers & Data** | `TabView` (multi-page navigation), `TileView` (2D swipeable grid), `Menu` (hierarchical settings), `List` (scrollable items), `Chart` (streaming waveform), `Table` (diagnostics grid), `Calendar` (date picker), `Canvas` (custom graphics) |

---

## 7. Developer Tutorial: Adding Your Own Feature

Here is how to add a new parameter and bind it from Rust to the display in 4 simple steps:

### Step 1: Declare a Parameter ID
In your application (e.g. [`crates/app-core/src/lib.rs`](file:///home/thughes/git/esp32-rust-lvgl-framework/crates/app-core/src/lib.rs)):
```rust
// Any application can declare custom parameter IDs:
pub const TARGET_TEMPERATURE: ParamId = ParamId::new(101);
```

### Step 2: Declare the Widget in Your Schema
In [`crates/app-core/src/lib.rs`](file:///home/thughes/git/esp32-rust-lvgl-framework/crates/app-core/src/lib.rs):
```rust
WidgetDescriptor::slider(
    ParamId::TargetTemperature,
    UiAlign::Center,
    0, 20,       // Center alignment with +20px Y offset
    15, 35, 21,  // 15°C to 35°C range, initial 21°C
    500,         // 500ms holdoff lease
    c"%ld C (Setting)",
    c"%ld C (Actual)",
).with_parent(ParamId::NavTab, 0),
```

### Step 3: Handle the Parameter in Rust Domain Logic
In `SystemEngine::process_command` ([`crates/app-core/src/lib.rs`](file:///home/thughes/git/esp32-rust-lvgl-framework/crates/app-core/src/lib.rs)):
```rust
ParamId::TargetTemperature => {
    let new_temp = unsafe { cmd.value.i32_val };
    self.setpoint_temp = new_temp;
}
```

### Step 4: Verify on Host, Then Flash to Hardware
```bash
# Preview changes instantly in offscreen test snapshots:
make -C test_harness run

# Flash to the physical board:
cargo build --release --target xtensa-esp32s3-none-elf -p app-core
idf.py -p /dev/ttyUSB0 flash
```

---

## 8. Hardware Deep Dive: Guition ESP32-S3-4848S040

The **Guition ESP32-S3-4848S040** is a popular 4.0-inch 480×480 smart display module.

```
       ┌────────────────────────────────────────────────────────┐
       │                 480x480 IPS Display                    │
       │                   (ST7701S RGB)                        │
       │                                                        │
       │       Controls   Selectors   Inputs   Data   Tools     │
       │     ┌────────────────────────────────────────────┐     │
       │     │                                            │     │
       │     │          Guition ESP32-S3-4848S040         │     │
       │     │                                            │     │
       │     └────────────────────────────────────────────┘     │
       │                                                        │
       └────────────────────────────────────────────────────────┘
```

### Display & Pinout Specifications
- **Display:** 4.0" IPS 480×480 RGB parallel interface driven by an Sitronix **ST7701S** controller.
- **Touch Screen:** Goodix **GT911** capacitive digitizer over I2C (`SDA: GPIO 19`, `SCL: GPIO 45`).
- **Memory:** 8MB Octal PSRAM (80MHz OPI), 16MB Quad SPI Flash (80MHz DIO).
- **Backlight Control:** `GPIO 38` (Active-High).

### Automatic 1-Minute Screen Timeout & Wake-on-Touch
To prevent LCD burn-in and minimize battery/power consumption:
- **Idle Screen Blanking:** After 60 seconds with no touch events, `screen_saver_timer_cb` turns off the LED backlight (`bsp_backlight_set(0)`).
- **Instant Wake-on-Touch:** When touched while dark, the GT911 touch interrupt restores the backlight (`bsp_backlight_set(100)`). The initial wake tap swallows the click event to prevent blind button presses.

### ST7701S RGB MDT (0xCD) Parallel Bus Fix
ST7701S panels on Guition 4848S040 displays require Command 2 Bank 0 register `0xCD` to be initialized to `0x00`. Without this command, the panel defaults to 18-bit sub-pixel mode, scrambling 16-bit RGB565 data into colored vertical stripes. This custom fix is integrated in [`main/bsp_4848s040.c`](file:///home/thughes/git/esp32-rust-lvgl-framework/main/bsp_4848s040.c#L101-L108).

---

## 9. Inter-Chip Link (ESP32-P4 + ESP32-C6)

On the **ESP32-P4**, wireless connectivity is delegated to an auxiliary **ESP32-C6** co-processor over **SDIO 3.0** or **High-Speed SPI** ([`main/include/p4_c6_link.h`](file:///home/thughes/git/esp32-rust-lvgl-framework/main/include/p4_c6_link.h)):

- **Multiplexed Virtual Channels:**
  - `P4_C6_CHAN_SYSTEM`: Reset, power management, clock synchronization.
  - `P4_C6_CHAN_WIFI`: 802.3 Ethernet frame encapsulation.
  - `P4_C6_CHAN_BLE`: Bluetooth 5.3 HCI commands and events.
  - `P4_C6_CHAN_802154`: Thread, Zigbee, and Matter packets.
  - `P4_C6_CHAN_TELEMETRY`: Co-processor RF telemetry and link metrics.
- **Packet Integrity:** Every frame is protected by a 16-bit sync magic (`0xE5P4`), sequence ticket, payload length, and CRC-CCITT checksum.

---

## 10. Automated Visual QA & Testing

The headless host test runner validates layouts, fonts, and state machines automatically. It renders frames into an offscreen RGB888 memory buffer and exports PNGs via `stb_image_write.h`.

```bash
cd test_harness
make run
```

### Snapshot Gallery (`test_harness/artifacts/`)

| Scenario | Snapshot | Description |
|---|---|---|
| **01** | `01_s3_tab0_controls_init.png` | S3-4848 Tab 0 (Controls): Stopped state, 10 Hz slider, 0 cycles |
| **02** | `02_s3_tab0_controls_running.png` | S3-4848 Tab 0 (Controls): RUNNING state, slider at 60 Hz, active cycles |
| **03** | `03_s3_tab1_selectors.png` | S3-4848 Tab 1 (Selectors): Dropdown, Roller, Spinbox & illuminated Aux LED |
| **04** | `04_s3_tab2_inputs_typing.png` | S3-4848 Tab 2 (Inputs): Virtual Keyboard focus retargeting & character entry |
| **05** | `05_s3_tab3_telemetry_waveforms.png` | S3-4848 Tab 3 (Telemetry): Real-time waveform oscilloscope & table registers |
| **06** | `06_s3_tab4_tools.png` | S3-4848 Tab 4 (Tools): Maintenance Calendar, 4×3 PIN Keypad & Canvas |
| **07** | `07_s3_tab5_system.png` | S3-4848 Tab 5 (System): Hierarchical Settings Menu & swipeable TileView |
| **08** | `08_s3_screen_blanked.png` | Power Management: 60-second inactivity timeout blanking display backlight |
| **09** | `09_s3_screen_woken.png` | Power Management: Wake-on-touch restoration with blind first-tap suppression |
| **10** | `10_tier1_showcase.png` | Isolated Showcase: All 12 Tier 1 scalar widgets active simultaneously |
| **11** | `11_tier2_showcase.png` | Isolated Showcase: Tier 2 inputs, virtual keyboard, button matrix & scale |
| **12** | `12_tier3_showcase.png` | Isolated Showcase: Tier 3 multi-tab container with charts, tables & lists |

---

## 11. Repository Structure
 
```text
esp32-rust-lvgl-framework/
├── CMakeLists.txt              # Top-level ESP-IDF CMake orchestration
├── .cargo/config.toml          # Xtensa Windowed ABI & bare-metal targets
├── Cargo.toml                  # Workspace definition (bridge-core, app-core)
├── components/                 # Reusable Modular ESP-IDF Components
│   ├── esp_lvgl_bridge/        # Core Bridge Engine Library Component
│   │   ├── CMakeLists.txt      # ESP-IDF component registration
│   │   ├── Kconfig             # Menuconfig settings (queue depth, holdoff, timeout)
│   │   ├── idf_component.yml   # ESP Component Registry manifest
│   │   ├── cmake/
│   │   │   └── esp_rust.cmake  # Reusable CMake helper for building & linking Rust staticlibs
│   │   ├── include/
│   │   │   ├── esp_lvgl_bridge.h # Umbrella public header
│   │   │   ├── bridge_protocol.h # C FFI mirror of bridge-core ABI
│   │   │   ├── ui_binding.h    # 26-widget schema parser & arbitration engine
│   │   │   └── p4_c6_link.h    # P4 <-> C6 co-processor link
│   │   └── src/
│   │       ├── bridge_transport.c # FreeRTOS ring queue message passing
│   │       ├── ui_binding.c    # Scene graph builder & arbitration engine
│   │       └── p4_c6_link.c    # SDIO 3.0 / High-Speed SPI driver
│   └── bsp_guition_s3_4848s040/ # Board Support Package Component
│       ├── CMakeLists.txt      # Registers ST7701S & GT911 dependencies
│       ├── idf_component.yml   # Managed component dependencies
│       ├── include/
│       │   └── bsp_4848s040.h  # Board HAL interface
│       └── src/
│           └── bsp_4848s040.c  # ST7701S RGB parallel bus & GT911 touch driver
├── crates/
│   ├── bridge-core/            # no_std Rust crate: ABI contracts & open ParamId newtype
│   └── app-core/               # Domain state machine & declarative UI schemas (staticlib)
├── main/                       # Application layer (consumes components)
│   ├── CMakeLists.txt          # Links esp_lvgl_bridge, BSP, and Rust via esp_rust.cmake
│   └── main.c                  # FreeRTOS dual-core task spawners & screensaver timer
├── docs/                       # Architecture specifications & mdBook guides
│   └── src/
│       ├── declarative_ui.md   # Complete widget catalog & schema API guide
│       └── ffi_contracts.md    # POD ring queue protocol specification
└── test_harness/               # Host-native headless visual test pipeline
    ├── Makefile                # Native host build (make run)
    ├── harness.c               # Offscreen RGB888 canvas & mock touch injector
    ├── main_test.c             # 12-scenario automated visual test runner
    └── artifacts/              # Generated visual PNG snapshots
```

---

## 12. License

- Framework codebase: Licensed under Apache-2.0 or MIT.
- [LVGL](https://lvgl.io): Licensed under MIT.
- `stb_image_write.h`: Public Domain / MIT.
