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
//! - [`EVERYTHING_SCHEMA`]: Complete 37-descriptor suite covering all 34 supported LVGL widgets across 7 tabs.
//! - [`TAB0_CONTROLS_SCHEMA`]: Tab 0 scalar controls (Switch, Slider, Arc, Bar, LED, Spinner, Button, Label, ArcLabel, ImageButton).
//! - [`TAB1_SELECTORS_SCHEMA`]: Tab 1 discrete selectors (Checkbox, Dropdown, Roller, Spinbox, Scale, AnimImage).
//! - [`TAB2_INPUTS_SCHEMA`]: Tab 2 text & input controls (TextInput, TextArea, Keyboard, MsgBox modal, SpanGroup).
//! - [`TAB3_TELEMETRY_SCHEMA`]: Tab 3 streaming telemetry (Oscilloscope Chart, Line separator, Diagnostics Table, List).
//! - [`TAB4_TOOLS_SCHEMA`]: Tab 4 interactive utilities (Calendar, ButtonMatrix keypad, Pixel Canvas).
//! - [`TAB5_MEDIA_SCHEMA`]: Tab 5 rich media & assets (Window frame, Barcode tag, Image preview).
//! - [`TAB6_SYSTEM_SCHEMA`]: Tab 6 system configuration (Hierarchical Menu, WiFi QR Code, Swipeable TileView).
//!
//! ## Exported C FFI Entry Points
//! - [`rust_worker_init`]: Allocates and initializes the static state engine.
//! - [`rust_worker_step`]: Advances the control loop by one tick (processes incoming commands, steps simulation).
//! - [`rust_get_everything_schema`]: Provides C UI engine with the complete 37-widget descriptor table.

#![cfg_attr(target_os = "none", no_std)]
#![allow(static_mut_refs)]

use core::mem::MaybeUninit;

use bridge_core::abi::{
    DeviceTelemetry, ParamId, ParamVal, UiCommand, UiAlign, WidgetDescriptor,
};

// ----------------------------------------------------------------------------
// Custom Domain Parameter Identifiers (Open ParamId Architecture)
// ----------------------------------------------------------------------------
pub const PARAM_SETPOINT_HZ: ParamId       = ParamId::new(1);
pub const PARAM_SYSTEM_ENABLE: ParamId     = ParamId::new(2);
pub const PARAM_TRIGGER_RESET: ParamId     = ParamId::new(3);
pub const PARAM_CYCLE_COUNT: ParamId       = ParamId::new(4);
pub const PARAM_ARC_KNOB: ParamId          = ParamId::new(5);
pub const PARAM_PROGRESS_BAR: ParamId      = ParamId::new(6);
pub const PARAM_AUX_CHECKBOX: ParamId      = ParamId::new(7);
pub const PARAM_STATUS_LED: ParamId        = ParamId::new(8);
pub const PARAM_BUSY_SPINNER: ParamId      = ParamId::new(9);
pub const PARAM_MODE_DROPDOWN: ParamId     = ParamId::new(10);
pub const PARAM_SPEED_ROLLER: ParamId      = ParamId::new(11);
pub const PARAM_PRECISION_STEP: ParamId    = ParamId::new(12);
pub const PARAM_TEXT_INPUT: ParamId        = ParamId::new(13);
pub const PARAM_KEYBOARD: ParamId          = ParamId::new(14);
pub const PARAM_KEYPAD_MATRIX: ParamId     = ParamId::new(15);
pub const PARAM_ALERT_MODAL: ParamId       = ParamId::new(16);
pub const PARAM_GAUGE_SCALE: ParamId       = ParamId::new(17);
pub const PARAM_NAV_TAB: ParamId           = ParamId::new(18);
pub const PARAM_NAV_TILE: ParamId          = ParamId::new(19);
pub const PARAM_MENU_NAV: ParamId          = ParamId::new(20);
pub const PARAM_ITEM_LIST: ParamId         = ParamId::new(21);
pub const PARAM_TELEMETRY_CHART: ParamId   = ParamId::new(22);
pub const PARAM_DATA_TABLE: ParamId        = ParamId::new(23);
pub const PARAM_CALENDAR_DATE: ParamId     = ParamId::new(24);
pub const PARAM_CANVAS_DRAW: ParamId       = ParamId::new(25);
pub const PARAM_TEXT_AREA: ParamId         = ParamId::new(26);
pub const PARAM_IMAGE_PREVIEW: ParamId     = ParamId::new(27);
pub const PARAM_LINE_GRAPH: ParamId        = ParamId::new(28);
pub const PARAM_ARC_LABEL_HEADING: ParamId = ParamId::new(29);
pub const PARAM_QR_WIFI_CONFIG: ParamId    = ParamId::new(30);
pub const PARAM_WINDOW_MODAL: ParamId      = ParamId::new(31);
pub const PARAM_IMAGE_TRIGGER: ParamId     = ParamId::new(32);
pub const PARAM_FORMATTED_SPAN: ParamId    = ParamId::new(33);
pub const PARAM_SPINNER_ANIM: ParamId      = ParamId::new(34);
pub const PARAM_ASSET_BARCODE: ParamId     = ParamId::new(35);

/// Root Navigation Container: 7-Tab TabView across 480x480 display
pub static NAV_CONTAINER_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::tabview(
        PARAM_NAV_TAB,
        c"Controls\nSelectors\nInputs\nTelemetry\nTools\nMedia\nSystem",
        480,
        480,
    ),
];

/// Tab 0: Controls (Core scalar controls & readouts)
pub static TAB0_CONTROLS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        PARAM_SYSTEM_ENABLE,
        UiAlign::TopLeft,
        20,
        15,
        c"System: RUNNING",
        c"System: STOPPED",
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::led(
        PARAM_STATUS_LED,
        UiAlign::TopRight,
        -20,
        18,
        true,
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::spinner(
        PARAM_BUSY_SPINNER,
        UiAlign::TopRight,
        -65,
        14,
        true,
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::arc(
        PARAM_ARC_KNOB,
        UiAlign::Center,
        0,
        -45,
        0,
        100,
        50,
        500,
        c"%ld%% (Setting)",
        c"%ld%% (Actual)",
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::arc_label(
        PARAM_ARC_LABEL_HEADING,
        UiAlign::Center,
        0,
        -45,
        82,
        245,
        50,
        c"POWER",
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::bar(
        PARAM_PROGRESS_BAR,
        UiAlign::Center,
        0,
        36,
        0,
        100,
        25,
        c"Load: %ld%%",
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::slider(
        PARAM_SETPOINT_HZ,
        UiAlign::Center,
        0,
        112,
        1,
        100,
        10,
        500,
        c"%ld Hz (Setting)",
        c"%ld Hz (Actual)",
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::button(
        PARAM_TRIGGER_RESET,
        UiAlign::BottomLeft,
        20,
        -15,
        c"Reset Counter",
        1,
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::image_button(
        PARAM_IMAGE_TRIGGER,
        UiAlign::BottomMid,
        0,
        -15,
        c"\xEF\x80\x91",
        1,
    ).with_parent(PARAM_NAV_TAB, 0),
    WidgetDescriptor::label(
        PARAM_CYCLE_COUNT,
        UiAlign::BottomRight,
        -20,
        -22,
        c"Cycles: %lu",
    ).with_parent(PARAM_NAV_TAB, 0),
];

/// Tab 1: Selectors (Discrete selectors & analog scale)
pub static TAB1_SELECTORS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::checkbox(
        PARAM_AUX_CHECKBOX,
        UiAlign::TopLeft,
        20,
        15,
        c"Auxiliary Motor",
        true,
    ).with_parent(PARAM_NAV_TAB, 1),
    WidgetDescriptor::dropdown(
        PARAM_MODE_DROPDOWN,
        UiAlign::TopLeft,
        20,
        60,
        c"Normal\nBoost\nEco\nSilent",
        0,
    ).with_parent(PARAM_NAV_TAB, 1),
    WidgetDescriptor::roller(
        PARAM_SPEED_ROLLER,
        UiAlign::TopLeft,
        20,
        135,
        c"Slow\nMedium\nFast\nUltra",
        1,
    ).with_parent(PARAM_NAV_TAB, 1),
    WidgetDescriptor::spinbox(
        PARAM_PRECISION_STEP,
        UiAlign::BottomLeft,
        25,
        -25,
        0,
        9999,
        1250,
    ).with_parent(PARAM_NAV_TAB, 1),
    WidgetDescriptor::scale(
        PARAM_GAUGE_SCALE,
        UiAlign::Center,
        105,
        10,
        210,
        210,
        0,
        100,
        65,
    ).with_parent(PARAM_NAV_TAB, 1),
    WidgetDescriptor::anim_image(
        PARAM_SPINNER_ANIM,
        UiAlign::TopRight,
        -35,
        20,
        1000,
    ).with_parent(PARAM_NAV_TAB, 1),
];

/// Tab 2: Inputs (Text inputs, virtual keyboard & alert modal)
pub static TAB2_INPUTS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::text_input(
        PARAM_TEXT_INPUT,
        UiAlign::TopMid,
        0,
        10,
        440,
        40,
        c"Single-line command...",
        Some(c"SYS:STATUS"),
        31,
    ).with_parent(PARAM_NAV_TAB, 2),
    WidgetDescriptor::text_area(
        PARAM_TEXT_AREA,
        UiAlign::TopMid,
        0,
        58,
        440,
        75,
        c"Multi-line notes...",
        Some(c"ESP32-S3 LVGL v9\nTouch input active."),
        false,
        127,
    ).with_parent(PARAM_NAV_TAB, 2),
    WidgetDescriptor::keyboard(
        PARAM_KEYBOARD,
        PARAM_TEXT_INPUT,
        UiAlign::BottomMid,
        0,
        -5,
        460,
        165,
    ).with_parent(PARAM_NAV_TAB, 2),
    WidgetDescriptor::msgbox(
        PARAM_ALERT_MODAL,
        c"System Notice",
        c"Hardware calibrated.",
        false,
    ),
    WidgetDescriptor::span_group(
        PARAM_FORMATTED_SPAN,
        UiAlign::TopMid,
        0,
        140,
        440,
        35,
        c"Console: |Online |Speed: |High",
    ).with_parent(PARAM_NAV_TAB, 2),
];

/// Tab 3: Telemetry (Streaming waveforms, diagnostics table & list)
pub static TAB3_TELEMETRY_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::chart(
        PARAM_TELEMETRY_CHART,
        PARAM_NAV_TAB,
        3,
        UiAlign::TopMid,
        0,
        8,
        430,
        120,
        0,
        100,
    ),
    WidgetDescriptor::line(
        PARAM_LINE_GRAPH,
        UiAlign::TopMid,
        0,
        140,
        430,
        4,
        Some(c"0,0; 430,0"),
    ).with_parent(PARAM_NAV_TAB, 3),
    WidgetDescriptor::table(
        PARAM_DATA_TABLE,
        PARAM_NAV_TAB,
        3,
        UiAlign::Center,
        0,
        48,
        430,
        105,
        (4 << 16) | 2,
        c"Module;Status;Core 0;OK;Core 1;OK;PSRAM;8MB",
    ),
    WidgetDescriptor::list(
        PARAM_ITEM_LIST,
        PARAM_NAV_TAB,
        3,
        UiAlign::BottomMid,
        0,
        -10,
        430,
        70,
        c"ST7701S RGB (Active)\nGT911 I2C (Ready)\nESP32-S3 (240MHz)",
    ),
];

/// Tab 4: Tools (Maintenance calendar, keypad matrix & pixel canvas)
pub static TAB4_TOOLS_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::calendar(
        PARAM_CALENDAR_DATE,
        PARAM_NAV_TAB,
        4,
        UiAlign::TopLeft,
        12,
        10,
        225,
        220,
    ),
    WidgetDescriptor::button_matrix(
        PARAM_KEYPAD_MATRIX,
        UiAlign::TopRight,
        -12,
        10,
        195,
        220,
        c"1 2 3 \n 4 5 6 \n 7 8 9 \n * 0 #",
    ).with_parent(PARAM_NAV_TAB, 4),
    WidgetDescriptor::canvas(
        PARAM_CANVAS_DRAW,
        PARAM_NAV_TAB,
        4,
        UiAlign::BottomLeft,
        20,
        -20,
        110,
        110,
    ),
    WidgetDescriptor::label(
        PARAM_CYCLE_COUNT,
        UiAlign::BottomRight,
        -20,
        -40,
        c"Calendar, Matrix & Canvas\nEngine Ticks: %lu",
    ).with_parent(PARAM_NAV_TAB, 4),
];

/// Tab 5: Media (Diagnostics window, barcode asset tag & image preview)
pub static TAB5_MEDIA_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::win(
        PARAM_WINDOW_MODAL,
        UiAlign::TopMid,
        0,
        10,
        440,
        180,
        c"Diagnostics Window",
    ).with_parent(PARAM_NAV_TAB, 5),
    WidgetDescriptor::barcode(
        PARAM_ASSET_BARCODE,
        UiAlign::BottomLeft,
        25,
        -25,
        180,
        70,
        c"ESP32-S3-4848",
    ).with_parent(PARAM_NAV_TAB, 5),
    WidgetDescriptor::image(
        PARAM_IMAGE_PREVIEW,
        UiAlign::BottomMid,
        50,
        -40,
        c"\xEF\x80\xBE",
    ).with_parent(PARAM_NAV_TAB, 5),
    WidgetDescriptor::label(
        PARAM_CYCLE_COUNT,
        UiAlign::BottomRight,
        -20,
        -35,
        c"Asset Tag & Icons\nFrames: %lu",
    ).with_parent(PARAM_NAV_TAB, 5),
];

/// Tab 6: System (Settings menu, WiFi onboarding QR code & swipeable tileview)
pub static TAB6_SYSTEM_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::menu(
        PARAM_MENU_NAV,
        UiAlign::TopLeft,
        15,
        10,
        250,
        180,
        c"System Settings",
    ).with_parent(PARAM_NAV_TAB, 6),
    WidgetDescriptor::qr_code(
        PARAM_QR_WIFI_CONFIG,
        UiAlign::TopRight,
        -15,
        15,
        140,
        c"WIFI:S:ESP32-S3-LAB;T:WPA;P:lvgl_rust;;",
    ).with_parent(PARAM_NAV_TAB, 6),
    WidgetDescriptor::tileview(
        PARAM_NAV_TILE,
        UiAlign::BottomMid,
        0,
        -10,
        440,
        180,
    ).with_parent(PARAM_NAV_TAB, 6),
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

static EVERYTHING_SCHEMA_ARRAY: [WidgetDescriptor; 37] = concat_schemas(&[
    NAV_CONTAINER_SCHEMA,
    TAB0_CONTROLS_SCHEMA,
    TAB1_SELECTORS_SCHEMA,
    TAB2_INPUTS_SCHEMA,
    TAB3_TELEMETRY_SCHEMA,
    TAB4_TOOLS_SCHEMA,
    TAB5_MEDIA_SCHEMA,
    TAB6_SYSTEM_SCHEMA,
]);

/// Complete 37-widget descriptor table joining all tab schemas and navigation container
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
            PARAM_SETPOINT_HZ => {
                self.target_hz = unsafe { cmd.value.i32_val };
            }
            PARAM_SYSTEM_ENABLE => {
                self.is_enabled = unsafe { cmd.value.bool_val };
                self.spinner_busy = self.is_enabled;
            }
            PARAM_TRIGGER_RESET => {
                self.cycle_counter = 0;
                self.progress_val = 0;
            }
            PARAM_CYCLE_COUNT => {}
            PARAM_ARC_KNOB => {
                self.arc_knob_val = unsafe { cmd.value.i32_val };
            }
            PARAM_PROGRESS_BAR => {}
            PARAM_AUX_CHECKBOX => {
                self.aux_enabled = unsafe { cmd.value.bool_val };
                self.led_state = self.aux_enabled;
            }
            PARAM_STATUS_LED => {}
            PARAM_BUSY_SPINNER => {}
            PARAM_MODE_DROPDOWN => {
                self.mode_dropdown_idx = unsafe { cmd.value.i32_val };
            }
            PARAM_SPEED_ROLLER => {
                self.speed_roller_idx = unsafe { cmd.value.i32_val };
            }
            PARAM_PRECISION_STEP => {
                self.spinbox_step_val = unsafe { cmd.value.i32_val };
            }
            PARAM_TEXT_INPUT => {
                self.text_buffer = unsafe { cmd.value.str_val };
            }
            PARAM_TEXT_AREA => {
                self.textarea_buffer = unsafe { cmd.value.str_val };
            }
            PARAM_KEYBOARD => {
                self.keyboard_action = unsafe { cmd.value.i32_val };
            }
            PARAM_KEYPAD_MATRIX => {
                self.keypad_last_key = unsafe { cmd.value.i32_val };
            }
            PARAM_ALERT_MODAL => {
                self.modal_open = unsafe { cmd.value.i32_val != 0 };
            }
            PARAM_GAUGE_SCALE => {
                self.gauge_val = unsafe { cmd.value.i32_val };
            }
            PARAM_NAV_TAB => {
                self.active_tab = unsafe { cmd.value.i32_val };
            }
            PARAM_CALENDAR_DATE => {
                self.selected_date = unsafe { cmd.value.u32_val };
            }
            ParamId::None
            | PARAM_NAV_TILE
            | PARAM_MENU_NAV
            | PARAM_ITEM_LIST
            | PARAM_TELEMETRY_CHART
            | PARAM_DATA_TABLE
            | PARAM_CANVAS_DRAW
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
            param_id: PARAM_SETPOINT_HZ,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.current_hz },
        };
        unsafe { bridge_send_telemetry(&telem_hz) };

        let telem_en = DeviceTelemetry {
            param_id: PARAM_SYSTEM_ENABLE,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.is_enabled },
        };
        unsafe { bridge_send_telemetry(&telem_en) };

        let telem_cnt = DeviceTelemetry {
            param_id: PARAM_CYCLE_COUNT,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { u32_val: self.cycle_counter },
        };
        unsafe { bridge_send_telemetry(&telem_cnt) };

        let telem_arc = DeviceTelemetry {
            param_id: PARAM_ARC_KNOB,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.arc_knob_val },
        };
        unsafe { bridge_send_telemetry(&telem_arc) };

        let telem_bar = DeviceTelemetry {
            param_id: PARAM_PROGRESS_BAR,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.progress_val },
        };
        unsafe { bridge_send_telemetry(&telem_bar) };

        let telem_aux = DeviceTelemetry {
            param_id: PARAM_AUX_CHECKBOX,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.aux_enabled },
        };
        unsafe { bridge_send_telemetry(&telem_aux) };

        let telem_led = DeviceTelemetry {
            param_id: PARAM_STATUS_LED,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.led_state },
        };
        unsafe { bridge_send_telemetry(&telem_led) };

        let telem_spin = DeviceTelemetry {
            param_id: PARAM_BUSY_SPINNER,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { bool_val: self.spinner_busy },
        };
        unsafe { bridge_send_telemetry(&telem_spin) };

        let telem_drop = DeviceTelemetry {
            param_id: PARAM_MODE_DROPDOWN,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.mode_dropdown_idx },
        };
        unsafe { bridge_send_telemetry(&telem_drop) };

        let telem_roll = DeviceTelemetry {
            param_id: PARAM_SPEED_ROLLER,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.speed_roller_idx },
        };
        unsafe { bridge_send_telemetry(&telem_roll) };

        let telem_step = DeviceTelemetry {
            param_id: PARAM_PRECISION_STEP,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.spinbox_step_val },
        };
        unsafe { bridge_send_telemetry(&telem_step) };

        let telem_scale = DeviceTelemetry {
            param_id: PARAM_GAUGE_SCALE,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.gauge_val },
        };
        unsafe { bridge_send_telemetry(&telem_scale) };

        let telem_chart = DeviceTelemetry {
            param_id: PARAM_TELEMETRY_CHART,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.chart_sample },
        };
        unsafe { bridge_send_telemetry(&telem_chart) };

        let telem_tab = DeviceTelemetry {
            param_id: PARAM_NAV_TAB,
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
        assert_eq!(EVERYTHING_SCHEMA.len(), 37);
        assert_eq!(TAB0_CONTROLS_SCHEMA.len(), 10);
        assert_eq!(TAB1_SELECTORS_SCHEMA.len(), 6);
        assert_eq!(TAB2_INPUTS_SCHEMA.len(), 5);
        assert_eq!(TAB3_TELEMETRY_SCHEMA.len(), 4);
        assert_eq!(TAB4_TOOLS_SCHEMA.len(), 4);
        assert_eq!(TAB5_MEDIA_SCHEMA.len(), 4);
        assert_eq!(TAB6_SYSTEM_SCHEMA.len(), 3);

        // Root widget of EVERYTHING schema must be TabView
        assert_eq!(EVERYTHING_SCHEMA[0].widget_type, WidgetType::TabView);
        assert_eq!(EVERYTHING_SCHEMA[0].param_id, PARAM_NAV_TAB);
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
            param_id: PARAM_SYSTEM_ENABLE,
            seq_id: 1,
            value: ParamVal::from_bool(true),
        });
        assert!(engine.is_enabled);
        assert!(engine.spinner_busy);

        // Request 15 Hz
        engine.process_command(UiCommand {
            param_id: PARAM_SETPOINT_HZ,
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
            param_id: PARAM_SYSTEM_ENABLE,
            seq_id: 1,
            value: ParamVal::from_bool(true),
        });
        for _ in 0..10 {
            engine.step();
        }
        assert_eq!(engine.cycle_counter, 10);

        // Trigger reset
        engine.process_command(UiCommand {
            param_id: PARAM_TRIGGER_RESET,
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
            param_id: PARAM_AUX_CHECKBOX,
            seq_id: 3,
            value: ParamVal::from_bool(true),
        });
        assert!(engine.aux_enabled);
        assert!(engine.led_state);

        // Mode dropdown
        engine.process_command(UiCommand {
            param_id: PARAM_MODE_DROPDOWN,
            seq_id: 4,
            value: ParamVal::from_i32(2),
        });
        assert_eq!(engine.mode_dropdown_idx, 2);

        // Tab navigation
        engine.process_command(UiCommand {
            param_id: PARAM_NAV_TAB,
            seq_id: 5,
            value: ParamVal::from_i32(3),
        });
        assert_eq!(engine.active_tab, 3);

        // Modal dialog toggle
        engine.process_command(UiCommand {
            param_id: PARAM_ALERT_MODAL,
            seq_id: 6,
            value: ParamVal::from_i32(1),
        });
        assert!(engine.modal_open);
    }
}
