# Building & Flashing Guide

## 1. Prerequisites
* **ESP-IDF**: v5.4+ or v5.5+
* **Python**: 3.10+ with `esptool`
* **Rust**: Toolchain supporting Xtensa (`xtensa-esp32s3-none-elf`) and RISC-V (`riscv32imafc-unknown-none-elf`)

## 2. Compiling Firmware

```bash
# Export the ESP-IDF toolchain environment
. $HOME/esp/esp-idf/export.sh

# Build for ESP32-S3 (compiles both Rust crates and ESP-IDF C components):
idf.py -DIDF_TARGET=esp32s3 build

# Or for ESP32-P4:
idf.py -DIDF_TARGET=esp32p4 build
```

## 3. Flashing to ESP32-S3-4848S040
Connect the device via the onboard USB-C port (identified as `/dev/ttyUSB0` or similar):

```bash
cd build
python -m esptool --chip esp32s3 -p /dev/ttyUSB0 -b 460800 \
    --before default_reset --after hard_reset \
    write_flash "@flash_args"
```

## 4. Serial Monitoring
```bash
idf.py -p /dev/ttyUSB0 monitor
```
Press `Ctrl + ]` to exit the monitor.
