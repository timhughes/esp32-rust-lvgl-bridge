# Hardware Platforms

The framework targets three primary Espressif chips with distinct roles:

| Target | Architecture | Core Freq | Memory Profile | Primary Display / Comm Peripherals | Role |
|---|---|---|---|---|---|
| **ESP32-S3** | Dual Xtensa LX7 | 240 MHz | 16MB Flash, 8MB Octal PSRAM | 16-bit Parallel RGB LCD, GT911 I2C | Smart Display & Edge Node |
| **ESP32-P4** | Dual RISC-V (HP) | 400 MHz | Up to 32MB PSRAM | MIPI-DSI, Hardware 2D PPA GPU, USB 2.0 HS | High-Performance Main Processor |
| **ESP32-C6** | Single RISC-V (LP) | 160 MHz | Internal SRAM | Wi-Fi 6, Bluetooth 5.3, 802.15.4 (Zigbee/Matter) | Low-Power Wireless Co-processor |
