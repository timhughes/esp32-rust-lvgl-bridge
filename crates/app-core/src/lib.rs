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
