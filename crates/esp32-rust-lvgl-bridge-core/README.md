# esp32-rust-lvgl-bridge-core

Zero-heap FFI ABI, Plain Old Data (POD) messaging, and declarative LVGL v9 UI schema contracts for Rust applications on ESP32 microcontrollers.

## Highlights

- **`no_std` Pure Rust**: Operates with zero heap allocations across the C/Rust boundary.
- **Fixed-Size POD Communication**: FreeRTOS queues pass UI commands and state telemetry by value.
- **Declarative UI Engine**: Define UI hierarchies, layout rules, and reactive data bindings with Rust data structures.
- **Closed-Loop Worker Trait**: Pre-defined traits for building reliable domain state machines.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
esp32-rust-lvgl-bridge-core = { version = "0.1.0" }
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
