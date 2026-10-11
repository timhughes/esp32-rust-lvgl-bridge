//! # Application Core: Domain Logic & Declarative UI Schemas
//!
//! This crate implements the core business logic, closed-loop state machines, and
//! declarative UI schemas for the **ESP32 Hybrid C/Rust LVGL Framework**.
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
//! - [`UI_SCHEMA`]: Baseline interactive schema (Switch, Slider, Reset Button, Cycle Counter).
//! - [`TIER1_SHOWCASE_SCHEMA`]: Comprehensive 12-widget layout covering all Tier 1 LVGL widgets.
//! - [`TIER2_SHOWCASE_SCHEMA`]: Text inputs, keypad button matrix, analog dial scale, and modal dialogs.
//! - [`TIER3_SHOWCASE_SCHEMA`]: Multi-tab container (`TabView`) routing charts, tables, calendar, lists, and canvas.
//! - [`S3_4848_SCHEMA`]: Hardware-tuned 480×480 layout for the Guition ESP32-S3 smart display.
//!
//! ## Exported C FFI Entry Points
//! - [`rust_worker_init`]: Allocates and initializes the static state engine.
//! - [`rust_worker_step`]: Advances the control loop by one tick (processes incoming commands, steps simulation).
//! - [`rust_get_ui_schema`]: Provides C UI engine with baseline interactive schema.
//! - [`rust_get_tier1_schema`]: Provides C UI engine with the 12-widget Tier 1 showcase schema.
//! - [`rust_get_tier2_schema`]: Provides C UI engine with the Tier 2 showcase schema.
//! - [`rust_get_tier3_schema`]: Provides C UI engine with the Tier 3 multi-tab showcase schema.
//! - [`rust_get_s3_schema`]: Provides C UI engine with the 480×480 S3 widget descriptor table.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(static_mut_refs)]

use core::mem::MaybeUninit;

use bridge_core::abi::{
    DeviceTelemetry, ParamId, ParamVal, UiCommand, UiAlign, WidgetDescriptor,
};

pub static UI_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        20,
        20,
        c"System: RUNNING",
        c"System: STOPPED",
    ),
    WidgetDescriptor::slider(
        ParamId::SetpointHz,
        UiAlign::Center,
        0,
        15,
        1,
        100,
        10,
        500,
        c"%ld Hz (Setting)",
        c"%ld Hz (Actual)",
    ),
    WidgetDescriptor::button(
        ParamId::TriggerReset,
        UiAlign::BottomLeft,
        20,
        -20,
        c"Reset Counter",
        1,
    ),
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomLeft,
        170,
        -30,
        c"Cycles: %lu",
    ),
];

pub static TIER1_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        30,
        25,
        c"System: RUNNING",
        c"System: STOPPED",
    ),
    WidgetDescriptor::checkbox(
        ParamId::AuxCheckbox,
        UiAlign::TopLeft,
        250,
        25,
        c"Aux Power",
        false,
    ),
    WidgetDescriptor::led(
        ParamId::StatusLed,
        UiAlign::TopLeft,
        420,
        25,
        true,
    ),
    WidgetDescriptor::spinner(
        ParamId::BusySpinner,
        UiAlign::TopLeft,
        500,
        18,
        true,
    ),
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
    WidgetDescriptor::dropdown(
        ParamId::ModeDropdown,
        UiAlign::Center,
        250,
        -70,
        c"Normal\nEco\nSport\nTurbo",
        0,
    ),
    WidgetDescriptor::roller(
        ParamId::SpeedRoller,
        UiAlign::Center,
        250,
        50,
        c"Slow\nMedium\nFast\nUltra",
        1,
    ),
    WidgetDescriptor::button(
        ParamId::TriggerReset,
        UiAlign::BottomLeft,
        40,
        -30,
        c"Reset Counter",
        1,
    ),
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomLeft,
        220,
        -38,
        c"Cycles: %lu",
    ),
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

pub static S3_4848_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        ParamId::SystemEnable,
        UiAlign::TopLeft,
        25,
        25,
        c"System: RUNNING",
        c"System: STOPPED",
    ),
    WidgetDescriptor::led(
        ParamId::StatusLed,
        UiAlign::TopRight,
        -35,
        28,
        true,
    ),
    WidgetDescriptor::spinner(
        ParamId::BusySpinner,
        UiAlign::TopRight,
        -85,
        22,
        true,
    ),
    WidgetDescriptor::arc(
        ParamId::ArcKnob,
        UiAlign::Center,
        0,
        -45,
        0,
        100,
        50,
        500,
        c"%ld%% (Setting)",
        c"%ld%% (Actual)",
    ),
    WidgetDescriptor::bar(
        ParamId::ProgressBar,
        UiAlign::Center,
        0,
        75,
        0,
        100,
        25,
        c"Load: %ld%%",
    ),
    WidgetDescriptor::button(
        ParamId::TriggerReset,
        UiAlign::BottomLeft,
        25,
        -25,
        c"Reset Counter",
        1,
    ),
    WidgetDescriptor::label(
        ParamId::CycleCount,
        UiAlign::BottomRight,
        -25,
        -35,
        c"Cycles: %lu",
    ),
];

pub static TIER2_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::text_area(
        ParamId::TextInput,
        UiAlign::TopMid,
        0,
        20,
        400,
        45,
        c"Enter command...",
        Some(c"SYS:STATUS"),
        true,
        31,
    ),
    WidgetDescriptor::button_matrix(
        ParamId::KeypadMatrix,
        UiAlign::Center,
        -160,
        -30,
        220,
        170,
        c"1 2 3 \n 4 5 6 \n 7 8 9 \n * 0 #",
    ),
    WidgetDescriptor::scale(
        ParamId::GaugeScale,
        UiAlign::Center,
        160,
        -30,
        180,
        180,
        0,
        100,
        65,
    ),
    WidgetDescriptor::keyboard(
        ParamId::Keyboard,
        ParamId::TextInput,
        UiAlign::BottomMid,
        0,
        -10,
        640,
        160,
    ),
    WidgetDescriptor::msgbox(
        ParamId::AlertModal,
        c"System Notice",
        c"Hardware calibrated.",
        false,
    ),
];

pub static TIER3_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::tabview(
        ParamId::NavTab,
        c"Telemetry\nDiagnostics\nConfig",
        760,
        460,
    ),
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

#[no_mangle]
pub extern "C" fn rust_get_ui_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = UI_SCHEMA.len() };
    }
    UI_SCHEMA.as_ptr()
}

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

#[no_mangle]
pub extern "C" fn rust_get_s3_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = S3_4848_SCHEMA.len() };
    }
    S3_4848_SCHEMA.as_ptr()
}

#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    pub fn bridge_recv_command(out_cmd: *mut UiCommand, timeout_ms: u32) -> bool;
    pub fn bridge_send_telemetry(telem: *const DeviceTelemetry) -> bool;
}

pub struct SystemEngine {
    target_hz: i32,
    current_hz: i32,
    is_enabled: bool,
    cycle_counter: u32,
    active_seq_id: u32,
    arc_knob_val: i32,
    progress_val: i32,
    aux_enabled: bool,
    led_state: bool,
    spinner_busy: bool,
    mode_dropdown_idx: i32,
    speed_roller_idx: i32,
    spinbox_step_val: i32,
    // Tier 2 & Tier 3 state
    text_buffer: [u8; 32],
    keyboard_action: i32,
    keypad_last_key: i32,
    modal_open: bool,
    gauge_val: i32,
    active_tab: i32,
    chart_sample: i32,
    selected_date: u32,
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
            | ParamId::CanvasDraw => {}
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
