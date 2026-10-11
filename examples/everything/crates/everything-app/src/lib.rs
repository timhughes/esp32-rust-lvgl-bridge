//! # Application Core: Domain Logic & Declarative UI Schemas
//!
//! This crate implements the core business logic, closed-loop state machines, and
//! declarative UI schemas for the **ESP32 Hybrid C/Rust LVGL Framework** Everything showcase.
//!
//! ## Architectural Principle: "C owns the display; Rust owns the system"
//!
//! - **C / ESP-IDF**: Drives display hardware (ST7701S RGB parallel bus, GT911 touch digitizer,
//!   FreeRTOS tasks, and the LVGL v9 scene graph).
//! - **Rust (`no_std`)**: Implements closed-loop state machines, domain models, protocol decoders,
//!   and deterministic hardware controllers.
//!
//! ## Declarative UI Schemas
//!
//! Instead of procedural C UI assembly, UI layouts are declared as static data tables:
//! - [`EVERYTHING_SCHEMA`]: Complete 27-descriptor suite covering all 25 supported LVGL widgets across 6 tabs.
//! - [`TAB0_CONTROLS_SCHEMA`]: Tab 0 scalar controls (Switch, Slider, Arc, Bar, LED, Spinner, Button, Label).
//! - [`TAB1_SELECTORS_SCHEMA`]: Tab 1 discrete selectors (Checkbox, Dropdown, Roller, Spinbox, Scale).
//! - [`TAB2_INPUTS_SCHEMA`]: Tab 2 text & input controls (TextInput, TextArea, Keyboard, MsgBox modal).
//! - [`TAB3_TELEMETRY_SCHEMA`]: Tab 3 streaming telemetry (Oscilloscope Chart, Diagnostics Table, List).
//! - [`TAB4_TOOLS_SCHEMA`]: Tab 4 interactive utilities (Calendar, ButtonMatrix keypad, Pixel Canvas).
//! - [`TAB5_SYSTEM_SCHEMA`]: Tab 5 system configuration (Hierarchical Menu, Swipeable TileView).
//!
//! ## Exported C FFI Entry Points
//! - [`rust_worker_init`]: Allocates and initializes the static state engine.
//! - [`rust_worker_step`]: Advances the control loop by one tick (processes incoming commands, steps simulation).
//! - [`rust_get_everything_schema`]: Provides C UI engine with the complete 27-widget descriptor table.

#![cfg_attr(target_os = "none", no_std)]
#![allow(static_mut_refs)]

use core::mem::MaybeUninit;

use bridge_core::abi::{
    DeviceTelemetry, ParamId, ParamVal, UiCommand, UiAlign, WidgetDescriptor,
};

/// Root Navigation Container: 6-Tab TabView across 480x480 display
pub static NAV_CONTAINER_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::tabview(
        ParamId::NavTab,
        c"Controls\nSelectors\nInputs\nTelemetry\nTools\nSystem",
        480,
        480,
    ),
];

/// Tab 0: Controls (Core scalar controls & readouts)
pub static TAB0_CONTROLS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        20,
        15,
        c"System: RUNNING",
        c"System: STOPPED",
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::led(
        ParamId::StatusLed,
        UiAlign::TopRight,
        -20,
        18,
        true,
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::spinner(
        ParamId::BusySpinner,
        UiAlign::TopRight,
        -65,
        14,
        true,
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::arc(
        ParamId::ArcKnob,
        UiAlign::Center,
        0,
        -55,
        0,
        100,
        50,
        500,
        c"%ld%% (Setting)",
        c"%ld%% (Actual)",
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::arc_label(
        ParamId::ArcLabelHeading,
        UiAlign::Center,
        0,
        -55,
        75,
        135,
        270,
        c"POWER",
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::bar(
        ParamId::ProgressBar,
        UiAlign::Center,
        0,
        45,
        0,
        100,
        25,
        c"Load: %ld%%",
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::slider(
        ParamId::SetpointHz,
        UiAlign::Center,
        0,
        95,
        1,
        100,
        10,
        500,
        c"%ld Hz (Setting)",
        c"%ld Hz (Actual)",
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::button(
        ParamId::TriggerReset,
        UiAlign::BottomLeft,
        20,
        -15,
        c"Reset Counter",
        1,
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::image_button(
        ParamId::ImageTrigger,
        UiAlign::BottomMid,
        0,
        -15,
        c"\xEF\x80\x91",
        1,
    ).with_parent(ParamId::NavTab, 0),
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomRight,
        -20,
        -22,
        c"Cycles: %lu",
    ).with_parent(ParamId::NavTab, 0),
];

/// Tab 1: Selectors (Discrete selectors & analog scale)
pub static TAB1_SELECTORS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::checkbox(
        ParamId::AuxCheckbox,
        UiAlign::TopLeft,
        20,
        15,
        c"Auxiliary Motor",
        true,
    ).with_parent(ParamId::NavTab, 1),
    WidgetDescriptor::dropdown(
        ParamId::ModeDropdown,
        UiAlign::TopLeft,
        20,
        60,
        c"Normal\nBoost\nEco\nSilent",
        0,
    ).with_parent(ParamId::NavTab, 1),
    WidgetDescriptor::roller(
        ParamId::SpeedRoller,
        UiAlign::TopLeft,
        20,
        135,
        c"Slow\nMedium\nFast\nUltra",
        1,
    ).with_parent(ParamId::NavTab, 1),
    WidgetDescriptor::spinbox(
        ParamId::PrecisionStep,
        UiAlign::BottomLeft,
        25,
        -25,
        0,
        9999,
        1250,
    ).with_parent(ParamId::NavTab, 1),
    WidgetDescriptor::scale(
        ParamId::GaugeScale,
        UiAlign::Center,
        105,
        10,
        210,
        210,
        0,
        100,
        65,
    ).with_parent(ParamId::NavTab, 1),
    WidgetDescriptor::anim_image(
        ParamId::SpinnerAnim,
        UiAlign::TopRight,
        -35,
        20,
        1000,
    ).with_parent(ParamId::NavTab, 1),
];

/// Tab 2: Inputs (Text inputs, virtual keyboard & alert modal)
pub static TAB2_INPUTS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::text_input(
        ParamId::TextInput,
        UiAlign::TopMid,
        0,
        10,
        440,
        40,
        c"Single-line command...",
        Some(c"SYS:STATUS"),
        31,
    ).with_parent(ParamId::NavTab, 2),
    WidgetDescriptor::text_area(
        ParamId::TextArea,
        UiAlign::TopMid,
        0,
        58,
        440,
        75,
        c"Multi-line notes...",
        Some(c"ESP32-S3 LVGL v9\nTouch input active."),
        false,
        127,
    ).with_parent(ParamId::NavTab, 2),
    WidgetDescriptor::keyboard(
        ParamId::Keyboard,
        ParamId::TextInput,
        UiAlign::BottomMid,
        0,
        -5,
        460,
        165,
    ).with_parent(ParamId::NavTab, 2),
    WidgetDescriptor::msgbox(
        ParamId::AlertModal,
        c"System Notice",
        c"Hardware calibrated.",
        false,
    ),
    WidgetDescriptor::span_group(
        ParamId::FormattedSpan,
        UiAlign::TopMid,
        0,
        140,
        440,
        35,
        c"Console: |Online |Speed: |High",
    ).with_parent(ParamId::NavTab, 2),
    WidgetDescriptor::win(
        ParamId::WindowModal,
        UiAlign::Center,
        0,
        0,
        320,
        220,
        c"Diagnostics Window",
    ).with_parent(ParamId::NavTab, 2),
];

/// Tab 3: Telemetry (Streaming waveforms, diagnostics table & list)
pub static TAB3_TELEMETRY_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::chart(
        ParamId::TelemetryChart,
        ParamId::NavTab,
        3,
        UiAlign::TopMid,
        0,
        10,
        420,
        150,
        0,
        100,
    ),
    WidgetDescriptor::line(
        ParamId::LineGraph,
        UiAlign::TopMid,
        0,
        95,
        420,
        2,
        Some(c"0,0; 420,0"),
    ).with_parent(ParamId::NavTab, 3),
    WidgetDescriptor::table(
        ParamId::DataTable,
        ParamId::NavTab,
        3,
        UiAlign::Center,
        0,
        35,
        420,
        120,
        (4 << 16) | 2,
        c"Module;Status;Core 0;OK;Core 1;OK;PSRAM;8MB",
    ),
    WidgetDescriptor::list(
        ParamId::ItemList,
        ParamId::NavTab,
        3,
        UiAlign::BottomMid,
        0,
        -15,
        420,
        75,
        c"ST7701S RGB (Active)\nGT911 I2C (Ready)\nESP32-S3 (240MHz)",
    ),
];

/// Tab 4: Tools (Maintenance calendar, keypad matrix & pixel canvas)
pub static TAB4_TOOLS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::calendar(
        ParamId::CalendarDate,
        ParamId::NavTab,
        4,
        UiAlign::TopLeft,
        12,
        10,
        225,
        220,
    ),
    WidgetDescriptor::button_matrix(
        ParamId::KeypadMatrix,
        UiAlign::TopRight,
        -12,
        10,
        195,
        220,
        c"1 2 3 \n 4 5 6 \n 7 8 9 \n * 0 #",
    ).with_parent(ParamId::NavTab, 4),
    WidgetDescriptor::canvas(
        ParamId::CanvasDraw,
        ParamId::NavTab,
        4,
        UiAlign::BottomLeft,
        20,
        -20,
        100,
        100,
    ),
    WidgetDescriptor::image(
        ParamId::ImagePreview,
        UiAlign::BottomRight,
        -130,
        -20,
        c"\xEF\x80\xBE",
    ).with_parent(ParamId::NavTab, 4),
    WidgetDescriptor::barcode(
        ParamId::AssetBarcode,
        UiAlign::BottomMid,
        0,
        -20,
        150,
        50,
        c"ESP32-S3-4848",
    ).with_parent(ParamId::NavTab, 4),
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomRight,
        -20,
        -45,
        c"Calendar, Matrix & Canvas\nEngine Ticks: %lu",
    ).with_parent(ParamId::NavTab, 4),
];

/// Tab 5: System (Settings menu, WiFi onboarding QR code & swipeable tileview)
pub static TAB5_SYSTEM_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::menu(
        ParamId::MenuNav,
        UiAlign::TopLeft,
        15,
        10,
        250,
        180,
        c"System Settings",
    ).with_parent(ParamId::NavTab, 5),
    WidgetDescriptor::qr_code(
        ParamId::QrWifiConfig,
        UiAlign::TopRight,
        -15,
        15,
        140,
        c"WIFI:S:ESP32-S3-LAB;T:WPA;P:lvgl_rust;;",
    ).with_parent(ParamId::NavTab, 5),
    WidgetDescriptor::tileview(
        ParamId::NavTile,
        UiAlign::BottomMid,
        0,
        -10,
        440,
        180,
    ).with_parent(ParamId::NavTab, 5),
];

const fn concat_schemas<const N: usize>(tables: &[&'static [WidgetDescriptor]]) -> [WidgetDescriptor; N] {
    let mut out = [tables[0][0]; N];
    let mut out_idx = 0;
    let mut t = 0;
    while t < tables.len() {
        let slice = tables[t];
        let mut i = 0;
        while i < slice.len() {
            out[out_idx] = slice[i];
            out_idx += 1;
            i += 1;
        }
        t += 1;
    }
    out
}

static EVERYTHING_SCHEMA_ARRAY: [WidgetDescriptor; 36] = concat_schemas(&[
    NAV_CONTAINER_SCHEMA,
    TAB0_CONTROLS_SCHEMA,
    TAB1_SELECTORS_SCHEMA,
    TAB2_INPUTS_SCHEMA,
    TAB3_TELEMETRY_SCHEMA,
    TAB4_TOOLS_SCHEMA,
    TAB5_SYSTEM_SCHEMA,
]);

/// Complete 36-widget descriptor table joining all tab schemas and navigation container
pub static EVERYTHING_SCHEMA: &[WidgetDescriptor] = &EVERYTHING_SCHEMA_ARRAY;

#[no_mangle]
pub extern "C" fn rust_get_everything_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = EVERYTHING_SCHEMA.len() };
    }
    EVERYTHING_SCHEMA.as_ptr()
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    pub fn bridge_recv_command(out_cmd: *mut UiCommand, timeout_ms: u32) -> bool;
    pub fn bridge_send_telemetry(telem: *const DeviceTelemetry) -> bool;
}

pub struct SystemEngine {
    pub target_hz: i32,
    pub current_hz: i32,
    pub is_enabled: bool,
    pub cycle_counter: u32,
    pub active_seq_id: u32,
    pub arc_knob_val: i32,
    pub progress_val: i32,
    pub aux_enabled: bool,
    pub led_state: bool,
    pub spinner_busy: bool,
    pub mode_dropdown_idx: i32,
    pub speed_roller_idx: i32,
    pub spinbox_step_val: i32,
    // Tier 2 & Tier 3 state
    pub text_buffer: [u8; 32],
    pub textarea_buffer: [u8; 32],
    pub keyboard_action: i32,
    pub keypad_last_key: i32,
    pub modal_open: bool,
    pub gauge_val: i32,
    pub active_tab: i32,
    pub chart_sample: i32,
    pub selected_date: u32,
}

impl SystemEngine {
    pub const fn new() -> Self {
        Self {
            target_hz: 10,
            current_hz: 10,
            is_enabled: false,
            cycle_counter: 0,
            active_seq_id: 0,
            arc_knob_val: 45,
            progress_val: 25,
            aux_enabled: false,
            led_state: true,
            spinner_busy: true,
            mode_dropdown_idx: 0,
            speed_roller_idx: 1,
            spinbox_step_val: 1250,
            text_buffer: [0u8; 32],
            textarea_buffer: [0u8; 32],
            keyboard_action: 0,
            keypad_last_key: 0,
            modal_open: false,
            gauge_val: 65,
            active_tab: 0,
            chart_sample: 30,
            selected_date: 20261011,
        }
    }

    pub fn process_command(&mut self, cmd: UiCommand) {
        self.active_seq_id = cmd.seq_id;
        match cmd.param_id {
            ParamId::SetpointHz => {
                self.target_hz = unsafe { cmd.value.i32_val };
            }
            ParamId::SystemEnable => {
                self.is_enabled = unsafe { cmd.value.bool_val };
                self.spinner_busy = self.is_enabled;
            }
            ParamId::TriggerReset => {
                self.cycle_counter = 0;
                self.progress_val = 0;
            }
            ParamId::CycleCount => {}
            ParamId::ArcKnob => {
                self.arc_knob_val = unsafe { cmd.value.i32_val };
            }
            ParamId::ProgressBar => {}
            ParamId::AuxCheckbox => {
                self.aux_enabled = unsafe { cmd.value.bool_val };
                self.led_state = self.aux_enabled;
            }
            ParamId::StatusLed => {}
            ParamId::BusySpinner => {}
            ParamId::ModeDropdown => {
                self.mode_dropdown_idx = unsafe { cmd.value.i32_val };
            }
            ParamId::SpeedRoller => {
                self.speed_roller_idx = unsafe { cmd.value.i32_val };
            }
            ParamId::PrecisionStep => {
                self.spinbox_step_val = unsafe { cmd.value.i32_val };
            }
            ParamId::TextInput => {
                self.text_buffer = unsafe { cmd.value.str_val };
            }
            ParamId::TextArea => {
                self.textarea_buffer = unsafe { cmd.value.str_val };
            }
            ParamId::Keyboard => {
                self.keyboard_action = unsafe { cmd.value.i32_val };
            }
            ParamId::KeypadMatrix => {
                self.keypad_last_key = unsafe { cmd.value.i32_val };
            }
            ParamId::AlertModal => {
                self.modal_open = unsafe { cmd.value.i32_val != 0 };
            }
            ParamId::GaugeScale => {
                self.gauge_val = unsafe { cmd.value.i32_val };
            }
            ParamId::NavTab => {
                self.active_tab = unsafe { cmd.value.i32_val };
            }
            ParamId::CalendarDate => {
                self.selected_date = unsafe { cmd.value.u32_val };
            }
            ParamId::None
            | ParamId::NavTile
            | ParamId::MenuNav
            | ParamId::ItemList
            | ParamId::TelemetryChart
            | ParamId::DataTable
            | ParamId::CanvasDraw
            | _ => {}
        }
    }

    pub fn step(&mut self) {
        if self.is_enabled {
            self.cycle_counter = self.cycle_counter.wrapping_add(1);

            if self.current_hz < self.target_hz {
                self.current_hz += 1;
            } else if self.current_hz > self.target_hz {
                self.current_hz -= 1;
            }

            self.progress_val = ((self.cycle_counter % 100) + 1) as i32;
            self.gauge_val = ((self.cycle_counter * 3) % 100) as i32;
            self.chart_sample = ((self.cycle_counter * 7 + 20) % 90 + 10) as i32;
        } else if self.current_hz > 0 {
            self.current_hz -= 1;
        }
    }

    pub fn emit_telemetry(&self) {
        let telem_hz = DeviceTelemetry {
            param_id: ParamId::SetpointHz,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.current_hz },
        };
        unsafe { bridge_send_telemetry(&telem_hz) };

        let telem_en = DeviceTelemetry {
            param_id: ParamId::SystemEnable,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.is_enabled },
        };
        unsafe { bridge_send_telemetry(&telem_en) };

        let telem_cnt = DeviceTelemetry {
            param_id: ParamId::CycleCount,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { u32_val: self.cycle_counter },
        };
        unsafe { bridge_send_telemetry(&telem_cnt) };

        let telem_arc = DeviceTelemetry {
            param_id: ParamId::ArcKnob,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.arc_knob_val },
        };
        unsafe { bridge_send_telemetry(&telem_arc) };

        let telem_bar = DeviceTelemetry {
            param_id: ParamId::ProgressBar,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.progress_val },
        };
        unsafe { bridge_send_telemetry(&telem_bar) };

        let telem_aux = DeviceTelemetry {
            param_id: ParamId::AuxCheckbox,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.aux_enabled },
        };
        unsafe { bridge_send_telemetry(&telem_aux) };

        let telem_led = DeviceTelemetry {
            param_id: ParamId::StatusLed,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.led_state },
        };
        unsafe { bridge_send_telemetry(&telem_led) };

        let telem_spin = DeviceTelemetry {
            param_id: ParamId::BusySpinner,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.spinner_busy },
        };
        unsafe { bridge_send_telemetry(&telem_spin) };

        let telem_drop = DeviceTelemetry {
            param_id: ParamId::ModeDropdown,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.mode_dropdown_idx },
        };
        unsafe { bridge_send_telemetry(&telem_drop) };

        let telem_roll = DeviceTelemetry {
            param_id: ParamId::SpeedRoller,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.speed_roller_idx },
        };
        unsafe { bridge_send_telemetry(&telem_roll) };

        let telem_step = DeviceTelemetry {
            param_id: ParamId::PrecisionStep,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.spinbox_step_val },
        };
        unsafe { bridge_send_telemetry(&telem_step) };

        let telem_scale = DeviceTelemetry {
            param_id: ParamId::GaugeScale,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.gauge_val },
        };
        unsafe { bridge_send_telemetry(&telem_scale) };

        let telem_chart = DeviceTelemetry {
            param_id: ParamId::TelemetryChart,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.chart_sample },
        };
        unsafe { bridge_send_telemetry(&telem_chart) };

        let telem_tab = DeviceTelemetry {
            param_id: ParamId::NavTab,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.active_tab },
        };
        unsafe { bridge_send_telemetry(&telem_tab) };
    }
}

static mut ENGINE: SystemEngine = SystemEngine::new();

#[no_mangle]
pub extern "C" fn rust_worker_init() {
    unsafe { ENGINE = SystemEngine::new(); }
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
    fn test_schemas_integrity() {
        assert_eq!(EVERYTHING_SCHEMA.len(), 36);
        assert_eq!(TAB0_CONTROLS_SCHEMA.len(), 10);
        assert_eq!(TAB1_SELECTORS_SCHEMA.len(), 6);
        assert_eq!(TAB2_INPUTS_SCHEMA.len(), 6);
        assert_eq!(TAB3_TELEMETRY_SCHEMA.len(), 4);
        assert_eq!(TAB4_TOOLS_SCHEMA.len(), 6);
        assert_eq!(TAB5_SYSTEM_SCHEMA.len(), 3);

        // Root widget of EVERYTHING schema must be TabView
        assert_eq!(EVERYTHING_SCHEMA[0].widget_type, WidgetType::TabView);
        assert_eq!(EVERYTHING_SCHEMA[0].param_id, ParamId::NavTab);
    }

    #[test]
    fn test_system_engine_initial_state() {
        let engine = SystemEngine::new();
        assert!(!engine.is_enabled);
        assert_eq!(engine.target_hz, 10);
        assert_eq!(engine.current_hz, 10);
        assert_eq!(engine.cycle_counter, 0);
        assert_eq!(engine.arc_knob_val, 45);
        assert_eq!(engine.progress_val, 25);
        assert_eq!(engine.mode_dropdown_idx, 0);
        assert_eq!(engine.speed_roller_idx, 1);
        assert_eq!(engine.spinbox_step_val, 1250);
        assert!(!engine.modal_open);
        assert_eq!(engine.active_tab, 0);
    }

    #[test]
    fn test_system_engine_control_loop_and_frequency_ramp() {
        let mut engine = SystemEngine::new();

        // Turn system ON
        engine.process_command(UiCommand {
            param_id: ParamId::SystemEnable,
            seq_id: 1,
            value: ParamVal::from_bool(true),
        });
        assert!(engine.is_enabled);
        assert!(engine.spinner_busy);

        // Request 15 Hz
        engine.process_command(UiCommand {
            param_id: ParamId::SetpointHz,
            seq_id: 2,
            value: ParamVal::from_i32(15),
        });
        assert_eq!(engine.target_hz, 15);

        // Step 5 times: current_hz ramps from 10 to 15
        for _ in 0..5 {
            engine.step();
        }
        assert_eq!(engine.current_hz, 15);
        assert_eq!(engine.cycle_counter, 5);
    }

    #[test]
    fn test_system_engine_reset_trigger() {
        let mut engine = SystemEngine::new();
        engine.process_command(UiCommand {
            param_id: ParamId::SystemEnable,
            seq_id: 1,
            value: ParamVal::from_bool(true),
        });
        for _ in 0..10 {
            engine.step();
        }
        assert_eq!(engine.cycle_counter, 10);

        // Trigger reset
        engine.process_command(UiCommand {
            param_id: ParamId::TriggerReset,
            seq_id: 2,
            value: ParamVal::from_i32(1),
        });
        assert_eq!(engine.cycle_counter, 0);
        assert_eq!(engine.progress_val, 0);
    }

    #[test]
    fn test_system_engine_discrete_and_navigation_commands() {
        let mut engine = SystemEngine::new();

        // Aux power checkbox synchronizes status LED
        engine.process_command(UiCommand {
            param_id: ParamId::AuxCheckbox,
            seq_id: 3,
            value: ParamVal::from_bool(true),
        });
        assert!(engine.aux_enabled);
        assert!(engine.led_state);

        // Mode dropdown
        engine.process_command(UiCommand {
            param_id: ParamId::ModeDropdown,
            seq_id: 4,
            value: ParamVal::from_i32(2),
        });
        assert_eq!(engine.mode_dropdown_idx, 2);

        // Tab navigation
        engine.process_command(UiCommand {
            param_id: ParamId::NavTab,
            seq_id: 5,
            value: ParamVal::from_i32(3),
        });
        assert_eq!(engine.active_tab, 3);

        // Modal dialog toggle
        engine.process_command(UiCommand {
            param_id: ParamId::AlertModal,
            seq_id: 6,
            value: ParamVal::from_i32(1),
        });
        assert!(engine.modal_open);
    }
}
