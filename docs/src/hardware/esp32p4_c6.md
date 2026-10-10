# ESP32-P4 & ESP32-C6 Co-Processor

On multi-chip hardware platforms, the high-performance **ESP32-P4** application processor connects to an **ESP32-C6** low-power wireless co-processor.

## Physical Interconnect
* **Bus**: High-Speed SDIO 3.0 (up to 50 MHz) or High-Speed Quad-SPI
* **Channel Multiplexing**:
  * `P4_C6_CHAN_MGMT`: Link synchronization, ping/pong, resets
  * `P4_C6_CHAN_WIFI`: Wi-Fi frames (802.11ax / Wi-Fi 6)
  * `P4_C6_CHAN_BLE`: HCI commands, events, and data packets (Bluetooth 5.3)
  * `P4_C6_CHAN_802154`: Thread, Zigbee, and Matter frames
  * `P4_C6_CHAN_TELEMETRY`: RF signal quality, RSSI, and health telemetry

## Framing & Integrity
Each frame on the bus follows a structured binary format:
```text
┌──────────────┬──────────────┬──────────────┬──────────────┬──────────────────┬──────────────┐
│ Magic (2B)   │ Chan ID (1B) │ Seq Ticket(1)│ Length (2B)  │  Payload (N B)   │ CRC16 (2B)   │
│ 0xE5P4       │ e.g. BLE     │ 0..255       │ 0..1500      │ Raw frame data   │ CRC-CCITT    │
└──────────────┴──────────────┴──────────────┴──────────────┴──────────────────┴──────────────┘
```
