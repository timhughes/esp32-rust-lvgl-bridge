# Everything Demonstration Application

The **Everything** application is the complete showcase of all 34 supported LVGL widgets in the **ESP32 Rust/LVGL Framework**, configured for the **Guition ESP32-S3-4848S040** 4.0-inch 480×480 smart display (and compatible with any ESP32-S3).

---

## Features Demonstrated

1. **Complete 34-Widget LVGL Suite Across 7 Tabs:**
   * **Tab 0 (Controls):** System Enable switch, LED indicator, activity spinner, arc knob, load progress bar, setpoint frequency slider with 3-stage arbitration, reset button, curved ArcLabel, ImageButton, and cycle readout.
   * **Tab 1 (Selectors):** Mode dropdown, speed roller, precision spinbox, analog Scale needle meter, Checkbox, and animated AnimImage.
   * **Tab 2 (Inputs):** Dynamic virtual on-screen QWERTY keyboard with automatic focus retargeting between single-line command fields and multi-line notes areas, modal MsgBox alerts, and multi-style SpanGroup text.
   * **Tab 3 (Telemetry):** Real-time sine waveform streaming oscilloscope chart, vector Line graph, multi-row system diagnostics status table (`Module`, `Status`, `Core 0`, `Core 1`, `PSRAM`), and interactive Item List.
   * **Tab 4 (Tools):** Maintenance Calendar date picker, 4×3 PIN security keypad matrix, and raw-pixel graphics Canvas with animated test pattern.
   * **Tab 5 (Media):** Movable modal Window frame, asset Barcode tag, and Image preview.
   * **Tab 6 (System):** Hierarchical settings menu, Wi-Fi configuration QR code, live hardware transport statistics readout (commands sent/received, telemetry sent/received), and swipeable 2D TileView navigation.

2. **Power Management & Inactivity Lifecycle:**
   * **Automatic 60s Blanking:** Blanks the display backlight (`bsp->set_backlight(0)`) after 60 seconds without touch interaction to prevent panel burn-in.
   * **Wake-on-Touch:** Tapping the dark screen restores full backlight (`bsp->set_backlight(100)`).
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
examples/everything/
├── CMakeLists.txt              # Standalone ESP-IDF project configuration
├── README.md                   # This guide
├── main/
│   ├── CMakeLists.txt          # Links esp_lvgl_bridge, BSP, and everything-app
│   └── main.c                  # C task initialization, screensaver timer & scene graph
└── crates/
    └── everything-app/         # no_std Rust application logic
        ├── Cargo.toml
        └── src/lib.rs          # 34-widget Everything schema across 7 tabs, SystemEngine & virtual keyboard wiring
```

---

## How to Build & Flash

```bash
# Using Makefile shortcut from repository root:
make build-everything
make flash-everything

# Or manually:
# 1. Build the Rust everything-app staticlib for Xtensa LX7:
cargo build --release --target xtensa-esp32s3-none-elf -p everything-app

# 2. Build and flash the ESP-IDF project:
cd examples/everything
idf.py -DIDF_TARGET=esp32s3 build
idf.py -p /dev/ttyUSB0 -b 460800 flash monitor
```
