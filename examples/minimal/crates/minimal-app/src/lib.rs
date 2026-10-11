//! # Minimal Starter Example: ESP32 Rust + LVGL Framework
//!
//! Demonstrates a minimal application defining custom parameter IDs,
//! declaring a 3-widget UI schema, and running a closed-loop control worker.

#![cfg_attr(target_os = "none", no_std)]
#![allow(static_mut_refs)]

#[cfg(target_os = "none")]
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
    pub const fn new() -> Self {
        Self {
            is_powered: false,
            target_speed: 25,
            current_speed: 25,
            cycle_count: 0,
            active_seq_id: 0,
        }
    }

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
        ENGINE = MinimalEngine::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_core::abi::{WidgetType, UiAlign};

    #[test]
    fn test_minimal_schema_integrity() {
        assert_eq!(MINIMAL_SCHEMA.len(), 3);

        // Widget 0: Power Switch
        assert_eq!(MINIMAL_SCHEMA[0].param_id, PARAM_POWER);
        assert_eq!(MINIMAL_SCHEMA[0].widget_type, WidgetType::Switch);
        assert_eq!(MINIMAL_SCHEMA[0].align, UiAlign::TopMid);

        // Widget 1: Speed Slider
        assert_eq!(MINIMAL_SCHEMA[1].param_id, PARAM_SPEED);
        assert_eq!(MINIMAL_SCHEMA[1].widget_type, WidgetType::Slider);
        assert_eq!(MINIMAL_SCHEMA[1].min_val, 0);
        assert_eq!(MINIMAL_SCHEMA[1].max_val, 100);
        assert_eq!(MINIMAL_SCHEMA[1].initial_val, 25);
        assert_eq!(MINIMAL_SCHEMA[1].holdoff_ms, 500);

        // Widget 2: Cycle Counter Label
        assert_eq!(MINIMAL_SCHEMA[2].param_id, PARAM_COUNTER);
        assert_eq!(MINIMAL_SCHEMA[2].widget_type, WidgetType::Label);
        assert_eq!(MINIMAL_SCHEMA[2].align, UiAlign::BottomMid);
    }

    #[test]
    fn test_minimal_engine_initial_state() {
        let engine = MinimalEngine::new();
        assert!(!engine.is_powered);
        assert_eq!(engine.target_speed, 25);
        assert_eq!(engine.current_speed, 25);
        assert_eq!(engine.cycle_count, 0);
    }

    #[test]
    fn test_minimal_engine_power_and_speed_control() {
        let mut engine = MinimalEngine::new();

        // Send power ON command
        let cmd_power = UiCommand {
            param_id: PARAM_POWER,
            seq_id: 1,
            value: ParamVal::from_bool(true),
        };
        engine.process_command(&cmd_power);
        assert!(engine.is_powered);
        assert_eq!(engine.active_seq_id, 1);

        // Send new target speed
        let cmd_speed = UiCommand {
            param_id: PARAM_SPEED,
            seq_id: 2,
            value: ParamVal::from_i32(30),
        };
        engine.process_command(&cmd_speed);
        assert_eq!(engine.target_speed, 30);
        assert_eq!(engine.current_speed, 25);

        // Step 5 times: speed should ramp by +1 each step
        for _ in 0..5 {
            engine.step();
        }
        assert_eq!(engine.current_speed, 30);
        assert_eq!(engine.cycle_count, 5);
    }

    #[test]
    fn test_minimal_engine_idle_when_powered_off() {
        let mut engine = MinimalEngine::new();
        assert!(!engine.is_powered);

        // When powered off, step does not advance cycles or speed
        engine.target_speed = 50;
        engine.step();
        assert_eq!(engine.cycle_count, 0);
        assert_eq!(engine.current_speed, 25);
    }
}
