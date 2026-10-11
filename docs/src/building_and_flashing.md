# Building & Flashing Guide

## 1. Prerequisites
* **ESP-IDF**: v5.4+ or v5.5+
* **Python**: 3.10+ (Python 3.13 recommended) with `esptool`
* **Rust**: Toolchain supporting Xtensa (`xtensa-esp32s3-none-elf`) and RISC-V (`riscv32imafc-unknown-none-elf`)

---

## 2. Compiling the Main Application (Flagship S3-4848)

```bash
# 1. Compile the bare-metal Xtensa Rust static library:
cargo build --release --target xtensa-esp32s3-none-elf -p s3-flagship-app

# 2. Export the ESP-IDF toolchain environment:
. $HOME/esp/esp-idf/export.sh

# 3. Build the ESP-IDF firmware binary:
idf.py -DIDF_TARGET=esp32s3 build
```

---

## 3. Compiling the Application Examples

### Minimal Starter (`examples/minimal`)
```bash
# 1. Compile the minimal Rust static library:
cargo build --release --target xtensa-esp32s3-none-elf -p minimal-app

# 2. Build the minimal ESP-IDF project:
cd examples/minimal
idf.py -DIDF_TARGET=esp32s3 build
```

### Widget Cookbook (`examples/widget_cookbook`)
```bash
# 1. Compile the cookbook Rust static library:
cargo build --release --target xtensa-esp32s3-none-elf -p widget-cookbook

# 2. Build the cookbook ESP-IDF project:
cd examples/widget_cookbook
idf.py -DIDF_TARGET=esp32s3 build
```

---

## 4. Flashing to Hardware (ESP32-S3-4848S040)

Connect the Guition device via the onboard USB-C port (`/dev/ttyUSB0`):

```bash
# Flash at high speed (460,800 baud):
idf.py -p /dev/ttyUSB0 -b 460800 flash

# Or using esptool directly:
esptool.py --chip esp32s3 -p /dev/ttyUSB0 -b 460800 \
    --before default_reset --after hard_reset \
    write_flash "@build/flash_args"
```

---

## 5. Serial Monitoring
```bash
idf.py -p /dev/ttyUSB0 monitor
```
Press `Ctrl + ]` to exit the serial monitor.
