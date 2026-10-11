# Flagship S3-4848 Demonstration Application

The **Flagship S3-4848** application is the reference implementation of the **ESP32 Rust/LVGL Framework** deployed on the **Guition ESP32-S3-4848S040** 4.0-inch 480×480 smart display.

---

## Features Demonstrated

1. **Complete 26-Widget LVGL Suite Across 6 Tabs:**
   * **Tab 0 (Controls):** System Enable switch, frequency setpoint slider with 3-stage arbitration, reset button, and cycle readout.
   * **Tab 1 (Selectors):** Mode dropdown, speed roller, precision spinbox, and auxiliary status LED.
   * **Tab 2 (Inputs):** Dynamic virtual on-screen QWERTY keyboard with automatic focus retargeting between single-line command fields and multi-line notes areas.
   * **Tab 3 (Telemetry):** Real-time sine waveform streaming oscilloscope chart and multi-row system diagnostics status table (`Module`, `Status`, `Core 0`, `Core 1`, `PSRAM`).
   * **Tab 4 (Tools):** Maintenance Calendar date picker, 4×3 PIN security keypad matrix, and raw-pixel graphics Canvas.
   * **Tab 5 (System):** Hierarchical settings menu and swipeable tileview navigation.

2. **Power Management & Inactivity Lifecycle:**
   * **Automatic 60s Blanking:** Blanks the display backlight (`bsp_backlight_set(0)`) after 60 seconds without touch interaction to prevent panel burn-in.
   * **Wake-on-Touch:** Tapping the dark screen restores full backlight (`bsp_backlight_set(100)`).
   * **Blind First-Tap Suppression:** Swallows the initial wake touch to prevent accidental button activation.

3. **Anti-Jitter 3-Stage Arbitration:**
   * Monotonic sequence tickets (`seq_id`) protect interactive widgets during active touch dragging (`ARB_DRAGGING`) and temporal holdoff (`ARB_HOLDOFF`), completely eliminating telemetry rubber-banding.

---

## Hardware Configuration (Guition ESP32-S3-4848S040)

* **Display Controller:** Sitronix **ST7701S** over 16-bit RGB parallel bus (Command 2 Bank 0 register `0xCD` configured to `0x00` to prevent 18-bit color distortion).
* **Touch Digitizer:** Goodix **GT911** capacitive touch controller over I2C (`SDA: GPIO 19`, `SCL: GPIO 45`).
* **Backlight:** `GPIO 38` (Active-High).
* **Memory:** 8MB Octal PSRAM (80MHz OPI), 16MB Quad SPI Flash.

---

## Project Structure

```text
examples/s3_4848_flagship/
├── CMakeLists.txt              # Standalone ESP-IDF project configuration
├── README.md                   # This guide
├── main/
│   ├── CMakeLists.txt          # Links esp_lvgl_bridge, BSP, and s3-flagship-app
│   └── main.c                  # C task initialization, screensaver timer & scene graph
└── crates/
    └── s3-flagship-app/        # no_std Rust application logic
        ├── Cargo.toml
        └── src/lib.rs          # 27-widget S3 schema, SystemEngine & virtual keyboard wiring
```

---

## How to Build & Flash

```bash
# 1. Build the Rust s3-flagship-app staticlib for Xtensa LX7:
cargo build --release --target xtensa-esp32s3-none-elf -p s3-flagship-app

# 2. Build and flash the standalone flagship project:
cd examples/s3_4848_flagship
idf.py -DIDF_TARGET=esp32s3 build
idf.py -p /dev/ttyUSB0 -b 460800 flash monitor
```
