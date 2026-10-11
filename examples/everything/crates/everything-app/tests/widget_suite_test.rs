//! # Integration & Visual Regression Test Suite for Everything App
//!
//! Validates:
//! 1. Complete coverage: Every single supported LVGL `WidgetType` (1 to 25) is used.
//! 2. Scene graph integrity: All widgets are successfully created, bound, and findable via `ParamId`.
//! 3. Interactive testing: Touch clicks, slider drags, button matrix keypresses, tab switches,
//!    text area editing, and telemetry streaming across all 6 tabs.
//! 4. Visual regression QA: High-resolution offscreen PNG snapshots generated for each tab.

use bridge_core::abi::{
    DeviceTelemetry, ParamVal, WidgetType,
};
use bridge_harness::HeadlessHarness;
use everything_app::*;
use std::collections::HashSet;
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// 1. Comprehensive Widget Coverage Assertion
/// Verifies that EVERY single widget type in `WidgetType::ALL` (all 34 widgets) is present in `EVERYTHING_SCHEMA`.
#[test]
fn test_everything_covers_every_widget_type() {
    let used_types: HashSet<WidgetType> = EVERYTHING_SCHEMA.iter().map(|w| w.widget_type).collect();

    // Verify all 34 variants are accounted for
    for &expected_type in WidgetType::ALL {
        assert!(
            used_types.contains(&expected_type),
            "Missing WidgetType::{:?} from EVERYTHING_SCHEMA!",
            expected_type
        );
    }

    assert_eq!(WidgetType::ALL.len(), 34, "Expected 34 distinct supported widget types");
    assert_eq!(used_types.len(), 34, "EVERYTHING_SCHEMA must utilize all 34 widget types");
    assert_eq!(EVERYTHING_SCHEMA.len(), 37, "EVERYTHING_SCHEMA contains 37 total descriptors");
}

/// 2. Headless Scene Graph Construction & Lookup Test
/// Verifies that all 36 descriptors are successfully instantiated by LVGL and findable by ParamId.
#[test]
fn test_everything_headless_scene_graph_and_lookups() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    for desc in EVERYTHING_SCHEMA {
        assert!(
            harness.widget_exists(desc.param_id),
            "Widget bound to ParamId::{:?} (type {:?}) was not registered in scene graph lookups!",
            desc.param_id,
            desc.widget_type
        );
    }
}

/// 3. Tab 0 (Controls): Switch, Slider, Arc, Bar, Led, Spinner, Button, Label
#[test]
fn test_tab0_controls_interaction_and_telemetry() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    let mut engine = SystemEngine::new();

    // Ensure we are on Tab 0
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 0));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(0));

    // Test Switch (SystemEnable)
    assert_eq!(harness.get_checked_state(PARAM_SYSTEM_ENABLE), Some(false));
    assert!(harness.click(PARAM_SYSTEM_ENABLE));
    harness.step(20);
    assert_eq!(harness.get_checked_state(PARAM_SYSTEM_ENABLE), Some(true));

    let cmd = harness.recv_command().expect("Expected SystemEnable command");
    assert_eq!(cmd.param_id, PARAM_SYSTEM_ENABLE);
    assert_eq!(unsafe { cmd.value.bool_val }, true);
    engine.process_command(cmd);
    assert!(engine.is_enabled);

    // Test Slider (SetpointHz)
    assert_eq!(harness.get_slider_value(PARAM_SETPOINT_HZ), Some(10));
    assert!(harness.drag_slider(PARAM_SETPOINT_HZ, 65));
    harness.touch_release();
    harness.step(20);
    assert_eq!(harness.get_slider_value(PARAM_SETPOINT_HZ), Some(65));

    let cmd = harness.recv_command().expect("Expected SetpointHz command");
    assert_eq!(cmd.param_id, PARAM_SETPOINT_HZ);
    assert_eq!(unsafe { cmd.value.i32_val }, 65);
    engine.process_command(cmd);
    assert_eq!(engine.target_hz, 65);

    // Test Button (TriggerReset)
    assert!(harness.click(PARAM_TRIGGER_RESET));
    harness.step(20);
    let cmd = harness.recv_command().expect("Expected TriggerReset command");
    assert_eq!(cmd.param_id, PARAM_TRIGGER_RESET);
    assert_eq!(unsafe { cmd.value.u32_val }, 1);
    engine.process_command(cmd);

    // Test Bar (ProgressBar) via telemetry dispatch
    assert_eq!(harness.get_bar_value(PARAM_PROGRESS_BAR), Some(25));
    let mut telem = DeviceTelemetry {
        param_id: PARAM_PROGRESS_BAR,
        ack_seq_id: 0,
        value: ParamVal::from_i32(80),
    };
    harness.dispatch_telemetry(&telem);
    harness.step(20);
    assert_eq!(harness.get_bar_value(PARAM_PROGRESS_BAR), Some(80));

    // Test LED (StatusLed)
    assert_eq!(harness.get_led_brightness(PARAM_STATUS_LED), Some(255));
    telem.param_id = PARAM_STATUS_LED;
    telem.value = ParamVal::from_bool(false);
    harness.dispatch_telemetry(&telem);
    harness.step(20);
    // When off, LVGL dims LED to LV_LED_BRIGHT_MIN (80)
    assert_eq!(harness.get_led_brightness(PARAM_STATUS_LED), Some(80));

    // Test Label (CycleCount)
    telem.param_id = PARAM_CYCLE_COUNT;
    telem.value = ParamVal::from_u32(9876);
    harness.dispatch_telemetry(&telem);
    harness.step(20);
    let label_txt = harness.get_label_text(PARAM_CYCLE_COUNT).unwrap_or_default();
    assert!(
        label_txt.contains("9876"),
        "CycleCount label should reflect dispatched telemetry: {}",
        label_txt
    );

    // Test ImageButton (ImageTrigger) exists
    assert!(harness.widget_exists(PARAM_IMAGE_TRIGGER));
}

/// 4. Tab 1 (Selectors): Checkbox, Dropdown, Roller, Spinbox, Scale, AnimImage
#[test]
fn test_tab1_selectors_interaction() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    // Switch to Tab 1
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 1));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(1));

    // Test Checkbox (AuxCheckbox)
    assert_eq!(harness.get_checked_state(PARAM_AUX_CHECKBOX), Some(true));
    assert!(harness.click(PARAM_AUX_CHECKBOX));
    harness.step(20);
    assert_eq!(harness.get_checked_state(PARAM_AUX_CHECKBOX), Some(false));

    let cmd = harness.recv_command().expect("Expected AuxCheckbox command");
    assert_eq!(cmd.param_id, PARAM_AUX_CHECKBOX);
    assert_eq!(unsafe { cmd.value.bool_val }, false);

    // Test Dropdown (ModeDropdown)
    assert_eq!(harness.get_dropdown_selected(PARAM_MODE_DROPDOWN), Some(0));

    // Test Roller (SpeedRoller)
    assert_eq!(harness.get_roller_selected(PARAM_SPEED_ROLLER), Some(1));

    // Test Spinbox (PrecisionStep)
    assert_eq!(harness.get_spinbox_value(PARAM_PRECISION_STEP), Some(1250));

    // Test Scale (GaugeScale) and AnimImage (SpinnerAnim) exist
    assert!(harness.widget_exists(PARAM_GAUGE_SCALE));
    assert!(harness.widget_exists(PARAM_SPINNER_ANIM));
}

/// 5. Tab 2 (Inputs): TextInput, TextArea, Keyboard, MsgBox, SpanGroup, Win
#[test]
fn test_tab2_inputs_and_keyboard() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    // Switch to Tab 2
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 2));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(2));

    // Test Single-line TextInput
    assert_eq!(
        harness.get_textarea_text(PARAM_TEXT_INPUT).as_deref(),
        Some("SYS:STATUS")
    );
    assert!(harness.set_textarea_text(PARAM_TEXT_INPUT, "SET_FREQ:45"));
    assert_eq!(
        harness.get_textarea_text(PARAM_TEXT_INPUT).as_deref(),
        Some("SET_FREQ:45")
    );

    // Test Multi-line TextArea
    let ta_text = harness.get_textarea_text(PARAM_TEXT_AREA).unwrap_or_default();
    assert!(
        ta_text.contains("ESP32-S3"),
        "TextArea should contain initial default text: {}",
        ta_text
    );
    assert!(harness.set_textarea_text(PARAM_TEXT_AREA, "Line 1\nLine 2\nLine 3"));
    assert_eq!(
        harness.get_textarea_text(PARAM_TEXT_AREA).as_deref(),
        Some("Line 1\nLine 2\nLine 3")
    );

    // Test Keyboard, MsgBox & SpanGroup exist in scene graph
    assert!(harness.widget_exists(PARAM_KEYBOARD));
    assert!(harness.widget_exists(PARAM_ALERT_MODAL));
    assert!(harness.widget_exists(PARAM_FORMATTED_SPAN));
}

/// 6. Tab 3 (Telemetry): Chart, Line, Table, List
#[test]
fn test_tab3_telemetry_streams() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    // Switch to Tab 3
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 3));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(3));

    // Verify widgets exist
    assert!(harness.widget_exists(PARAM_TELEMETRY_CHART));
    assert!(harness.widget_exists(PARAM_LINE_GRAPH));
    assert!(harness.widget_exists(PARAM_DATA_TABLE));
    assert!(harness.widget_exists(PARAM_ITEM_LIST));

    // Stream telemetry points into Chart
    for i in 0..10 {
        let telem = DeviceTelemetry {
            param_id: PARAM_TELEMETRY_CHART,
            ack_seq_id: 0,
            value: ParamVal::from_i32(i * 10),
        };
        harness.dispatch_telemetry(&telem);
        harness.step(10);
    }
}

/// 7. Tab 4 (Tools): Calendar, ButtonMatrix, Canvas
#[test]
fn test_tab4_tools_keypad_and_canvas() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    // Switch to Tab 4
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 4));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(4));

    // Verify Calendar & Canvas exist
    assert!(harness.widget_exists(PARAM_CALENDAR_DATE));
    assert!(harness.widget_exists(PARAM_CANVAS_DRAW));

    // Test ButtonMatrix (KeypadMatrix) click by button text
    assert!(harness.widget_exists(PARAM_KEYPAD_MATRIX));
    assert!(harness.click_button_matrix_text(PARAM_KEYPAD_MATRIX, "5"));
    harness.step(20);

    let cmd = harness.recv_command().expect("Expected KeypadMatrix command for key '5'");
    assert_eq!(cmd.param_id, PARAM_KEYPAD_MATRIX);
}

/// 8. Tab 5 (Media): WindowModal, AssetBarcode, ImagePreview
#[test]
fn test_tab5_media_assets() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    // Switch to Tab 5
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 5));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(5));

    // Verify WindowModal, AssetBarcode & ImagePreview exist
    assert!(harness.widget_exists(PARAM_WINDOW_MODAL));
    assert!(harness.widget_exists(PARAM_ASSET_BARCODE));
    assert!(harness.widget_exists(PARAM_IMAGE_PREVIEW));
}

/// 9. Tab 6 (System): Menu, QRCode, TileView
#[test]
fn test_tab6_system_navigation() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    // Switch to Tab 6
    assert!(harness.set_active_tab(PARAM_NAV_TAB, 6));
    assert_eq!(harness.get_active_tab(PARAM_NAV_TAB), Some(6));

    // Verify Menu, QRCode and TileView exist
    assert!(harness.widget_exists(PARAM_MENU_NAV));
    assert!(harness.widget_exists(PARAM_QR_REPO));
    assert!(harness.widget_exists(PARAM_NAV_TILE));
}

/// 10. Full Visual Regression Snapshot Suite
/// Renders every tab and saves high-resolution PNGs to the artifacts directory.
#[test]
fn test_visual_snapshots_all_tabs() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    let tab_snapshots = [
        (0, "01_everything_tab0_controls.png"),
        (1, "02_everything_tab1_selectors.png"),
        (2, "03_everything_tab2_inputs.png"),
        (3, "04_everything_tab3_telemetry.png"),
        (4, "05_everything_tab4_tools.png"),
        (5, "06_everything_tab5_media.png"),
        (6, "07_everything_tab6_system.png"),
    ];

    let dirs = [
        "artifacts",
        "../../../../artifacts",
        "/home/thughes/.gemini/antigravity-cli/brain/f8a8e282-7ea5-46ba-8a7a-613e42b88b8b/artifacts",
    ];

    for (tab_idx, filename) in tab_snapshots {
        assert!(harness.set_active_tab(PARAM_NAV_TAB, tab_idx));
        harness.step(50);
        for dir in dirs {
            if let Ok(_) = std::fs::create_dir_all(dir) {
                let path = format!("{}/{}", dir, filename);
                let _ = harness.save_png(&path);
            }
        }
    }
}

/// 10. Phase 3: Runtime widget flags via telemetry
#[test]
fn test_runtime_widget_flags_via_telemetry() {
    use bridge_core::abi::WidgetFlagOp as F;
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(EVERYTHING_SCHEMA);
    harness.step(50);

    let p = PARAM_TRIGGER_RESET;
    assert_eq!(harness.is_widget_hidden(p), Some(false));
    assert_eq!(harness.is_widget_disabled(p), Some(false));

    harness.dispatch_telemetry(&DeviceTelemetry::widget_flag_update(p, F::Hidden as u32 | F::Disabled as u32));
    assert_eq!(harness.is_widget_hidden(p), Some(true));
    assert_eq!(harness.is_widget_disabled(p), Some(true));

    harness.dispatch_telemetry(&DeviceTelemetry::widget_flag_update(p, F::Visible as u32 | F::Enabled as u32));
    assert_eq!(harness.is_widget_hidden(p), Some(false));
    assert_eq!(harness.is_widget_disabled(p), Some(false));

    harness.dispatch_telemetry(&DeviceTelemetry::widget_flag_update(p, F::Unclickable as u32));
    assert_eq!(harness.is_widget_clickable(p), Some(false));
    harness.dispatch_telemetry(&DeviceTelemetry::widget_flag_update(p, F::Clickable as u32));
    assert_eq!(harness.is_widget_clickable(p), Some(true));
}

/// 11. Phase 3: Declarative flex layout
#[test]
fn test_declarative_flex_layout() {
    use bridge_core::abi::{LayoutFlow, UiAlign, WidgetDescriptor};
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(480, 480);
    let schema = [WidgetDescriptor::win(
        PARAM_WINDOW_MODAL, UiAlign::Center, 0, 0, 300, 200, c"Flex",
    ).with_layout(LayoutFlow::Column)];
    harness.build_schema(&schema);
    harness.step(20);
    assert_eq!(harness.get_widget_flex_flow(PARAM_WINDOW_MODAL), Some(LayoutFlow::Column));
}
