//! # Headless LVGL Test Harness for Rust
//!
//! Provides idiomatic Rust bindings to the headless LVGL software renderer,
//! in-memory transport queues, and mock touch injector.
//!
//! Enables embedded developers to write 100% pure Rust unit and visual
//! regression tests for their declarative UI schemas without physical hardware.

use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::path::Path;

use esp32_rust_lvgl_bridge_core::abi::{
    DeviceTelemetry, ParamId, UiCommand, WidgetDescriptor,
};

#[allow(dead_code)]
extern "C" {
    fn bridge_transport_init(cmd_queue_len: usize, telem_queue_len: usize);
    fn bridge_send_command_from_ui(cmd: *const UiCommand) -> bool;
    fn bridge_recv_command(out_cmd: *mut UiCommand, timeout_ms: u32) -> bool;
    fn bridge_send_telemetry(telem: *const DeviceTelemetry) -> bool;
    fn bridge_recv_telemetry_in_ui(out_telem: *mut DeviceTelemetry) -> bool;

    fn harness_init();
    fn harness_set_resolution(hor_res: i32, ver_res: i32);
    fn harness_step(elapsed_ms: u32);
    fn harness_step_loop(duration_ms: u32, step_interval_ms: u32);
    fn harness_touch_set(x: i32, y: i32, pressed: bool);
    fn harness_touch_release();
    fn harness_click_widget(obj: *mut c_void);
    fn harness_drag_slider_to_val(slider: *mut c_void, target_val: i32);
    fn harness_save_png(filename: *const c_char) -> bool;
    fn harness_get_hor_res() -> i32;
    fn harness_get_ver_res() -> i32;
    fn bsp_backlight_is_on() -> bool;

    fn ui_engine_init();
    fn ui_engine_build_from_schema(schema: *const WidgetDescriptor, count: usize);
    fn ui_engine_get_widget_by_param(param_id: u32) -> *mut c_void;
    fn ui_engine_dispatch_telemetry(telem: *const DeviceTelemetry);
    fn ui_engine_pump_telemetry();

    fn lv_slider_get_value(obj: *const c_void) -> i32;
    fn lv_obj_has_state(obj: *const c_void, state: u32) -> bool;
    fn lv_dropdown_get_selected(obj: *const c_void) -> u32;
    fn lv_roller_get_selected(obj: *const c_void) -> u32;
    fn lv_spinbox_get_value(obj: *const c_void) -> i32;
    fn lv_arc_get_value(obj: *const c_void) -> i32;
    fn lv_bar_get_value(obj: *const c_void) -> i32;
}

/// Headless LVGL test harness managing offscreen framebuffer and input injection.
pub struct HeadlessHarness {
    _private: (),
}

impl HeadlessHarness {
    /// Initialize the headless offscreen LVGL display and in-memory bridge transports.
    pub fn new(hor_res: i32, ver_res: i32) -> Self {
        unsafe {
            bridge_transport_init(32, 64);
            harness_init();
            harness_set_resolution(hor_res, ver_res);
            ui_engine_init();
        }
        Self { _private: () }
    }

    /// Set or update the canvas resolution.
    pub fn set_resolution(&mut self, hor_res: i32, ver_res: i32) {
        unsafe { harness_set_resolution(hor_res, ver_res) };
    }

    /// Dynamically construct the LVGL UI scene graph from a declarative schema.
    pub fn build_schema(&mut self, schema: &[WidgetDescriptor]) {
        unsafe { ui_engine_build_from_schema(schema.as_ptr(), schema.len()) };
    }

    /// Advance LVGL tick and process timers by `elapsed_ms`.
    pub fn step(&mut self, elapsed_ms: u32) {
        unsafe { harness_step(elapsed_ms) };
    }

    /// Pump pending telemetry from the in-memory queue to bound widgets.
    pub fn pump_telemetry(&mut self) {
        let mut telem = std::mem::MaybeUninit::<DeviceTelemetry>::uninit();
        unsafe {
            while bridge_recv_telemetry_in_ui(telem.as_mut_ptr()) {
                ui_engine_dispatch_telemetry(telem.as_ptr());
            }
        }
    }

    /// Dispatch a single telemetry metric directly to the UI engine.
    pub fn dispatch_telemetry(&mut self, telem: &DeviceTelemetry) {
        unsafe { ui_engine_dispatch_telemetry(telem) };
    }

    /// Pop the next command emitted by user interaction.
    pub fn recv_command(&mut self) -> Option<UiCommand> {
        let mut cmd = std::mem::MaybeUninit::<UiCommand>::uninit();
        if unsafe { bridge_recv_command(cmd.as_mut_ptr(), 0) } {
            Some(unsafe { cmd.assume_init() })
        } else {
            None
        }
    }

    /// Simulate a touch press and release (click) on a widget bound to `param_id`.
    pub fn click(&mut self, param_id: ParamId) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            return false;
        }
        unsafe { harness_click_widget(widget) };
        true
    }

    /// Simulate dragging a slider bound to `param_id` to `target_val`.
    pub fn drag_slider(&mut self, param_id: ParamId, target_val: i32) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            return false;
        }
        unsafe { harness_drag_slider_to_val(widget, target_val) };
        true
    }

    /// Lift touch at the current coordinate.
    pub fn touch_release(&mut self) {
        unsafe { harness_touch_release() };
    }

    /// Set raw touch coordinate and pressed state.
    pub fn touch_set(&mut self, x: i32, y: i32, pressed: bool) {
        unsafe { harness_touch_set(x, y, pressed) };
    }

    /// Save the current offscreen framebuffer to a PNG image.
    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path_str = path.as_ref().to_str().ok_or("Invalid path string")?;
        let c_path = CString::new(path_str).map_err(|e| e.to_string())?;
        let ok = unsafe { harness_save_png(c_path.as_ptr()) };
        if ok {
            Ok(())
        } else {
            Err(format!("Failed to save screenshot to {}", path_str))
        }
    }

    /// Query the current value of a slider widget.
    pub fn get_slider_value(&self, param_id: ParamId) -> Option<i32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_slider_get_value(widget) })
        }
    }

    /// Query the checked/toggled state of a switch or checkbox (LV_STATE_CHECKED = 1).
    pub fn get_checked_state(&self, param_id: ParamId) -> Option<bool> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_obj_has_state(widget, 0x0001) })
        }
    }

    /// Query the current selected index of a dropdown widget.
    pub fn get_dropdown_selected(&self, param_id: ParamId) -> Option<u32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_dropdown_get_selected(widget) })
        }
    }

    /// Query the current selected index of a roller widget.
    pub fn get_roller_selected(&self, param_id: ParamId) -> Option<u32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_roller_get_selected(widget) })
        }
    }

    /// Query the current value of an arc widget.
    pub fn get_arc_value(&self, param_id: ParamId) -> Option<i32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_arc_get_value(widget) })
        }
    }

    /// Query the current value of a spinbox widget.
    pub fn get_spinbox_value(&self, param_id: ParamId) -> Option<i32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.0) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_spinbox_get_value(widget) })
        }
    }

    /// Check if the simulated display backlight is currently active.
    pub fn backlight_is_on(&self) -> bool {
        unsafe { bsp_backlight_is_on() }
    }
}
