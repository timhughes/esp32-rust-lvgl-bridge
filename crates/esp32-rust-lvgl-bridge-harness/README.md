# esp32-rust-lvgl-bridge-harness

Headless LVGL host test harness for running pure Rust integration, unit, and pixel-perfect visual QA regression tests on developer workstations and CI runners.

## Highlights

- **Pure Rust Testing**: Write UI tests using standard `cargo test` without C test runners or custom Makefiles.
- **Headless Software Engine**: Drives LVGL v9 headlessly in host memory at native CPU speeds.
- **Visual Regression Testing**: Captures framebuffer snapshots to PNG format (`save_png`) for automated visual assertions.
- **Event Simulation**: Inject touch events (`touch_down`, `touch_up`, `touch_move`) and settle the UI event loop with simulated time progression (`step_ms`).

## Usage

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
esp32-rust-lvgl-bridge-harness = { version = "0.1.0" }
```

In your test file (`tests/ui_test.rs`):

```rust
use esp32_rust_lvgl_bridge_harness::HeadlessHarness;

#[test]
fn test_my_ui() {
    let mut harness = HeadlessHarness::new(480, 480);
    harness.mount_ui(&MY_UI_SCHEMA);
    harness.step_ms(100);
    harness.save_png("artifacts/initial.png").unwrap();
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
