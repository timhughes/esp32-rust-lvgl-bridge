//! # Minimal Starter Example: ESP32 Rust + LVGL Framework
//!
//! Demonstrates a minimal application defining custom parameter IDs,
//! declaring a 3-widget UI schema, and running a closed-loop control worker.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(static_mut_refs)]

#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use bridge_core::abi::{
    DeviceTelemetry, ParamId, ParamVal, UiAlign, UiCommand, WidgetDescriptor,
};

// ----------------------------------------------------------------------------
// 1. Custom Domain Parameter Identifiers (Open ParamId Architecture)
// ----------------------------------------------------------------------------
pub const PARAM_POWER: ParamId   = ParamId::new(101);
pub const PARAM_SPEED: ParamId   = ParamId::new(102);
pub const PARAM_COUNTER: ParamId = ParamId::new(103);

// ----------------------------------------------------------------------------
// 2. Declarative UI Schema
// ----------------------------------------------------------------------------
pub static MINIMAL_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor::switch(
        PARAM_POWER,
        UiAlign::TopMid,
        0,
        40,
        c"Power: ON",
        c"Power: OFF",
    ),
    WidgetDescriptor::slider(
        PARAM_SPEED,
        UiAlign::Center,
        0,
        -10,
        0,
        100,
        25,
        500,
        c"Speed: %ld RPM (Setting)",
        c"Speed: %ld RPM (Actual)",
    ),
    WidgetDescriptor::label(
        PARAM_COUNTER,
        UiAlign::BottomMid,
        0,
        -40,
        c"Cycles: %ld",
    ),
];

// ----------------------------------------------------------------------------
// 3. Domain State Machine
// ----------------------------------------------------------------------------
pub struct MinimalEngine {
    pub is_powered: bool,
    pub target_speed: i32,
    pub current_speed: i32,
    pub cycle_count: u32,
    pub active_seq_id: u32,
}

static mut ENGINE: MinimalEngine = MinimalEngine {
    is_powered: false,
    target_speed: 25,
    current_speed: 25,
    cycle_count: 0,
    active_seq_id: 0,
};

extern "C" {
    fn bridge_recv_command(out_cmd: *mut UiCommand, timeout_ms: u32) -> bool;
    fn bridge_send_telemetry(telem: *const DeviceTelemetry) -> bool;
}

impl MinimalEngine {
    pub fn process_command(&mut self, cmd: &UiCommand) {
        self.active_seq_id = cmd.seq_id;
        match cmd.param_id {
            PARAM_POWER => {
                self.is_powered = unsafe { cmd.value.bool_val };
            }
            PARAM_SPEED => {
                self.target_speed = unsafe { cmd.value.i32_val };
            }
            _ => {}
        }
    }

    pub fn step(&mut self) {
        if self.is_powered {
            self.cycle_count = self.cycle_count.wrapping_add(1);

            // Smoothly approach target speed
            if self.current_speed < self.target_speed {
                self.current_speed += 1;
            } else if self.current_speed > self.target_speed {
                self.current_speed -= 1;
            }
        }
    }

    pub fn emit_telemetry(&self) {
        let telem_speed = DeviceTelemetry {
            param_id: PARAM_SPEED,
            ack_seq_id: self.active_seq_id,
            value: ParamVal { i32_val: self.current_speed },
        };
        unsafe { bridge_send_telemetry(&telem_speed) };

        let telem_count = DeviceTelemetry {
            param_id: PARAM_COUNTER,
            ack_seq_id: 0,
            value: ParamVal { i32_val: self.cycle_count as i32 },
        };
        unsafe { bridge_send_telemetry(&telem_count) };
    }
}

// ----------------------------------------------------------------------------
// 4. Exported C FFI Entry Points
// ----------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn rust_get_minimal_schema(out_count: *mut usize) -> *const WidgetDescriptor {
    if !out_count.is_null() {
        unsafe { *out_count = MINIMAL_SCHEMA.len() };
    }
    MINIMAL_SCHEMA.as_ptr()
}

#[no_mangle]
pub extern "C" fn rust_minimal_init() {
    unsafe {
        ENGINE = MinimalEngine {
            is_powered: false,
            target_speed: 25,
            current_speed: 25,
            cycle_count: 0,
            active_seq_id: 0,
        };
    }
}

#[no_mangle]
pub extern "C" fn rust_minimal_step() {
    unsafe {
        let mut cmd = core::mem::MaybeUninit::<UiCommand>::uninit();
        while bridge_recv_command(cmd.as_mut_ptr(), 0) {
            let valid_cmd = cmd.assume_init();
            ENGINE.process_command(&valid_cmd);
        }

        ENGINE.step();
        ENGINE.emit_telemetry();
    }
}
