//! Integration Test for esp32-rust-lvgl-bridge-harness
//!
//! Validates headless offscreen canvas initialization, schema construction,
//! user input emulation, and PNG framebuffer export in pure Rust.

use esp32_rust_lvgl_bridge_core::abi::{ParamId, UiAlign, WidgetDescriptor};
use esp32_rust_lvgl_bridge_harness::HeadlessHarness;

const PARAM_SWITCH: ParamId = ParamId::new(1);
const PARAM_SLIDER: ParamId = ParamId::new(2);

static TEST_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        PARAM_SWITCH,
        UiAlign::TopLeft,
        20,
        20,
        c"State: ON",
        c"State: OFF",
    ),
    WidgetDescriptor::slider(
        PARAM_SLIDER,
        UiAlign::Center,
        0,
        0,
        0,
        100,
        50,
        500,
        c"Value: %ld (Setting)",
        c"Value: %ld (Active)",
    ),
];

#[test]
fn test_headless_harness_lifecycle() {
    std::fs::create_dir_all("artifacts").unwrap();

    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(TEST_SCHEMA);
    harness.step(50);

    // Verify initial values (switch starts un-checked, slider at 50)
    assert_eq!(harness.get_checked_state(PARAM_SWITCH), Some(false));
    assert_eq!(harness.get_slider_value(PARAM_SLIDER), Some(50));

    // Toggle switch via touch click
    assert!(harness.click(PARAM_SWITCH));
    harness.step(20);
    assert_eq!(harness.get_checked_state(PARAM_SWITCH), Some(true));

    // Interact with slider
    assert!(harness.drag_slider(PARAM_SLIDER, 75));
    harness.touch_release();
    harness.step(20);
    assert_eq!(harness.get_slider_value(PARAM_SLIDER), Some(75));

    // Save offscreen rendering to PNG
    harness.save_png("artifacts/smoke_test_render.png").unwrap();
}
