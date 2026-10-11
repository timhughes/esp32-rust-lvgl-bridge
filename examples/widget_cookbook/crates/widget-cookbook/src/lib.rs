//! # Widget Cookbook & Declarative UI Reference
//!
//! A comprehensive cookbook demonstrating how to define, layout, and bind
//! all 27 supported LVGL widgets using `bridge-core`'s declarative descriptors.
//!
//! This crate serves as a reference recipe book for developers building
//! applications with the **ESP32 Hybrid C/Rust LVGL Framework**.
//!
//! ## Recipe Index
//!
//! - **Recipe 1: Tier 1 Scalar Controls** ([`TIER1_SHOWCASE_SCHEMA`]):
//!   Switches, sliders, arcs, progress bars, checkboxes, LEDs, spinners,
//!   dropdowns, rollers, buttons, labels, and spinboxes.
//! - **Recipe 2: Tier 2 Inputs & Keypads** ([`TIER2_SHOWCASE_SCHEMA`]):
//!   Single-line text inputs, multi-line text areas, virtual keyboards,
//!   button matrix numeric keypads, analog dial scales, and modal dialogs.
//! - **Recipe 3: Tier 3 Containers & Visualizations** ([`TIER3_SHOWCASE_SCHEMA`]):
//!   Multi-tab containers (`TabView`), streaming waveforms (`Chart`), structured
//!   data grids (`Table`), interactive calendars, scrollable lists, menus, and canvas.

#![cfg_attr(target_os = "none", no_std)]
#![allow(static_mut_refs)]

use core::mem::MaybeUninit;

use bridge_core::abi::{
    DeviceTelemetry, ParamId, ParamVal, UiAlign, UiCommand, WidgetDescriptor,
};

// ======================================================================
// Recipe 1: Tier 1 Scalar Controls & Indicators (12 Widgets)
// ======================================================================

pub static TIER1_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    // 1. Two-State Toggle Switch
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        30,
        25,
        c"System: RUNNING",
        c"System: STOPPED",
    ),
    // 2. Binary Checkbox
    WidgetDescriptor::checkbox(
        ParamId::AuxCheckbox,
        UiAlign::TopLeft,
        250,
        25,
        c"Aux Power",
        false,
    ),
    // 3. Status LED (Bistable On/Off)
    WidgetDescriptor::led(
        ParamId::StatusLed,
        UiAlign::TopLeft,
        420,
        25,
        true,
    ),
    // 4. Activity Spinner
    WidgetDescriptor::spinner(
        ParamId::BusySpinner,
        UiAlign::TopLeft,
        500,
        18,
        true,
    ),
    // 5. Circular Arc Gauge / Knob
    WidgetDescriptor::arc(
        ParamId::ArcKnob,
        UiAlign::Center,
        -250,
        -30,
        0,
        100,
        45,
        500,
        c"%ld%% (Setting)",
        c"%ld%% (Actual)",
    ),
    // 6. Linear Slider
    WidgetDescriptor::slider(
        ParamId::SetpointHz,
        UiAlign::Center,
        0,
        -50,
        1,
        100,
        10,
        500,
        c"%ld Hz (Setting)",
        c"%ld Hz (Actual)",
    ),
    // 7. Horizontal Progress Bar
    WidgetDescriptor::bar(
        ParamId::ProgressBar,
        UiAlign::Center,
        0,
        30,
        0,
        100,
        25,
        c"Progress: %ld%%",
    ),
    // 8. Dropdown Selection List
    WidgetDescriptor::dropdown(
        ParamId::ModeDropdown,
        UiAlign::Center,
        250,
        -70,
        c"Normal\nEco\nSport\nTurbo",
        0,
    ),
    // 9. Vertical Roller Selector
    WidgetDescriptor::roller(
        ParamId::SpeedRoller,
        UiAlign::Center,
        250,
        50,
        c"Slow\nMedium\nFast\nUltra",
        1,
    ),
    // 10. Momentary Action Button
    WidgetDescriptor::button(
        ParamId::TriggerReset,
        UiAlign::BottomLeft,
        40,
        -30,
        c"Reset Counter",
        1,
    ),
    // 11. Dynamic Telemetry Label
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomLeft,
        220,
        -38,
        c"Cycles: %lu",
    ),
    // 12. Numeric Precision Step Box
    WidgetDescriptor::spinbox(
        ParamId::PrecisionStep,
        UiAlign::BottomRight,
        -60,
        -30,
        0,
        9999,
        1250,
    ),
];

// ======================================================================
// Recipe 2: Tier 2 Text, Keypads, Scales & Modals
// ======================================================================

pub static TIER2_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    // 1. Single-Line Text Input Field
    WidgetDescriptor::text_input(
        ParamId::TextInput,
        UiAlign::TopLeft,
        40,
        15,
        340,
        42,
        c"Single-line command...",
        Some(c"SYS:STATUS"),
        31,
    ),
    // 2. Multi-Line Scrollable Text Area
    WidgetDescriptor::text_area(
        ParamId::TextArea,
        UiAlign::TopRight,
        -40,
        15,
        340,
        75,
        c"Multi-line notes...",
        Some(c"System log initialized.\nReady for commands."),
        false,
        127,
    ),
    // 3. Button Matrix Keypad
    WidgetDescriptor::button_matrix(
        ParamId::KeypadMatrix,
        UiAlign::Center,
        -180,
        -15,
        220,
        150,
        c"1 2 3 \n 4 5 6 \n 7 8 9 \n * 0 #",
    ),
    // 4. Analog Scale Dial
    WidgetDescriptor::scale(
        ParamId::GaugeScale,
        UiAlign::Center,
        180,
        -15,
        180,
        180,
        0,
        100,
        65,
    ),
    // 5. On-Screen Virtual Keyboard
    WidgetDescriptor::keyboard(
        ParamId::Keyboard,
        ParamId::TextInput,
        UiAlign::BottomMid,
        0,
        -10,
        720,
        160,
    ),
    // 6. Modal Message Alert Box
    WidgetDescriptor::msgbox(
        ParamId::AlertModal,
        c"System Notice",
        c"Hardware calibrated successfully.",
        false,
    ),
];

// ======================================================================
// Recipe 3: Tier 3 Multi-Tab Containers & Visualizations
// ======================================================================

pub static TIER3_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    // Root Multi-Tab Container
    WidgetDescriptor::tabview(
        ParamId::NavTab,
        c"Telemetry\nDiagnostics\nConfig",
        760,
        460,
    ),
    // Tab 0: Telemetry Waveform Chart
    WidgetDescriptor::chart(
        ParamId::TelemetryChart,
        ParamId::NavTab,
        0,
        UiAlign::Center,
        0,
        -70,
        640,
        220,
        0,
        100,
    ),
    // Tab 0: Scrollable Channel Status List
    WidgetDescriptor::list(
        ParamId::ItemList,
        ParamId::NavTab,
        0,
        UiAlign::BottomMid,
        0,
        -15,
        640,
        110,
        c"Channel A (Active)\nChannel B (Standby)\nBus Voltage (3.3V)",
    ),
    // Tab 1: System Diagnostic Table (4 rows x 2 columns)
    WidgetDescriptor::table(
        ParamId::DataTable,
        ParamId::NavTab,
        1,
        UiAlign::Center,
        0,
        0,
        500,
        240,
        (4 << 16) | 2,
        c"Module;Status;Core 0;OK;Core 1;OK;PSRAM;8MB",
    ),
    // Tab 2: Interactive Date Picker Calendar
    WidgetDescriptor::calendar(
        ParamId::CalendarDate,
        ParamId::NavTab,
        2,
        UiAlign::Center,
        0,
        0,
        320,
        280,
    ),
];

// ======================================================================
// C FFI Schema Accessors
// ======================================================================

#[no_mangle]
pub extern "C" fn rust_get_tier1_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = TIER1_SHOWCASE_SCHEMA.len() };
    }
    TIER1_SHOWCASE_SCHEMA.as_ptr()
}

#[no_mangle]
pub extern "C" fn rust_get_tier2_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = TIER2_SHOWCASE_SCHEMA.len() };
    }
    TIER2_SHOWCASE_SCHEMA.as_ptr()
}

#[no_mangle]
pub extern "C" fn rust_get_tier3_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = TIER3_SHOWCASE_SCHEMA.len() };
    }
    TIER3_SHOWCASE_SCHEMA.as_ptr()
}

// ======================================================================
// Simulation & Hardware Loop
// ======================================================================

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    pub fn bridge_recv_command(out_cmd: *mut UiCommand, timeout_ms: u32) -> bool;
    pub fn bridge_send_telemetry(telem: *const DeviceTelemetry) -> bool;
}

pub struct CookbookEngine {
    is_enabled: bool,
    counter: u32,
    setpoint_hz: i32,
    active_seq_id: u32,
}

impl CookbookEngine {
    pub const fn new() -> Self {
        Self {
            is_enabled: true,
            counter: 0,
            setpoint_hz: 25,
            active_seq_id: 0,
        }
    }

    pub fn process_command(&mut self, cmd: UiCommand) {
        self.active_seq_id = cmd.seq_id;
        match cmd.param_id {
            ParamId::SystemEnable => {
                self.is_enabled = unsafe { cmd.value.bool_val };
            }
            ParamId::SetpointHz => {
                self.setpoint_hz = unsafe { cmd.value.i32_val };
            }
            ParamId::TriggerReset => {
                self.counter = 0;
            }
            _ => {}
        }
    }

    pub fn step(&mut self) {
        if self.is_enabled {
            self.counter = self.counter.wrapping_add(1);
        }
    }

    pub fn emit_telemetry(&self) {
        let telem_cnt = DeviceTelemetry {
            param_id: ParamId::CycleCount,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { u32_val: self.counter },
        };
        unsafe { bridge_send_telemetry(&telem_cnt) };

        let telem_hz = DeviceTelemetry {
            param_id: ParamId::SetpointHz,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.setpoint_hz },
        };
        unsafe { bridge_send_telemetry(&telem_hz) };
    }
}

static mut ENGINE: CookbookEngine = CookbookEngine::new();

#[no_mangle]
pub extern "C" fn rust_worker_init() {
    unsafe { ENGINE = CookbookEngine::new(); }
}

#[no_mangle]
pub extern "C" fn rust_worker_step() {
    let mut cmd_buf = MaybeUninit::<UiCommand>::uninit();

    while unsafe { bridge_recv_command(cmd_buf.as_mut_ptr(), 0) } {
        let cmd = unsafe { cmd_buf.assume_init() };
        unsafe { ENGINE.process_command(cmd) };
    }

    unsafe {
        ENGINE.step();
        ENGINE.emit_telemetry();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_core::abi::WidgetType;

    #[test]
    fn test_cookbook_schemas_integrity() {
        assert_eq!(TIER1_SHOWCASE_SCHEMA.len(), 12);
        assert_eq!(TIER2_SHOWCASE_SCHEMA.len(), 6);
        assert_eq!(TIER3_SHOWCASE_SCHEMA.len(), 5);

        // Verify Tier 1 schema widgets
        assert_eq!(TIER1_SHOWCASE_SCHEMA[0].widget_type, WidgetType::Switch);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[1].widget_type, WidgetType::Checkbox);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[2].widget_type, WidgetType::Led);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[3].widget_type, WidgetType::Spinner);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[4].widget_type, WidgetType::Arc);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[5].widget_type, WidgetType::Slider);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[6].widget_type, WidgetType::Bar);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[7].widget_type, WidgetType::Dropdown);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[8].widget_type, WidgetType::Roller);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[9].widget_type, WidgetType::Button);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[10].widget_type, WidgetType::Label);
        assert_eq!(TIER1_SHOWCASE_SCHEMA[11].widget_type, WidgetType::Spinbox);

        // Verify Tier 2 schema widgets
        assert_eq!(TIER2_SHOWCASE_SCHEMA[0].widget_type, WidgetType::TextArea);
        assert_eq!(TIER2_SHOWCASE_SCHEMA[1].widget_type, WidgetType::TextArea);
        assert_eq!(TIER2_SHOWCASE_SCHEMA[2].widget_type, WidgetType::ButtonMatrix);
        assert_eq!(TIER2_SHOWCASE_SCHEMA[3].widget_type, WidgetType::Scale);
        assert_eq!(TIER2_SHOWCASE_SCHEMA[4].widget_type, WidgetType::Keyboard);
        assert_eq!(TIER2_SHOWCASE_SCHEMA[5].widget_type, WidgetType::MsgBox);

        // Verify Tier 3 schema widgets
        assert_eq!(TIER3_SHOWCASE_SCHEMA[0].widget_type, WidgetType::TabView);
        assert_eq!(TIER3_SHOWCASE_SCHEMA[1].widget_type, WidgetType::Chart);
        assert_eq!(TIER3_SHOWCASE_SCHEMA[2].widget_type, WidgetType::List);
        assert_eq!(TIER3_SHOWCASE_SCHEMA[3].widget_type, WidgetType::Table);
        assert_eq!(TIER3_SHOWCASE_SCHEMA[4].widget_type, WidgetType::Calendar);
    }

    #[test]
    fn test_cookbook_engine_lifecycle() {
        let mut engine = CookbookEngine::new();
        assert!(engine.is_enabled);
        assert_eq!(engine.counter, 0);
        assert_eq!(engine.setpoint_hz, 25);

        // Step 10 times
        for _ in 0..10 {
            engine.step();
        }
        assert_eq!(engine.counter, 10);

        // Set frequency to 60 Hz
        engine.process_command(UiCommand {
            param_id: ParamId::SetpointHz,
            seq_id: 1,
            value: ParamVal::from_i32(60),
        });
        assert_eq!(engine.setpoint_hz, 60);

        // Reset counter
        engine.process_command(UiCommand {
            param_id: ParamId::TriggerReset,
            seq_id: 2,
            value: ParamVal::from_i32(1),
        });
        assert_eq!(engine.counter, 0);

        // Disable engine
        engine.process_command(UiCommand {
            param_id: ParamId::SystemEnable,
            seq_id: 3,
            value: ParamVal::from_bool(false),
        });
        assert!(!engine.is_enabled);

        // Step while disabled: counter should remain 0
        engine.step();
        assert_eq!(engine.counter, 0);
    }
}
