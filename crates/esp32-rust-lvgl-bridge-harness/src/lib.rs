//! # Headless LVGL Test Harness for Rust
//!
//! Provides idiomatic Rust bindings to the headless LVGL software renderer,
//! in-memory transport queues, and mock touch injector.
//!
//! Enables embedded developers to write 100% pure Rust unit and visual
//! regression tests for their declarative UI schemas without physical hardware.
//!
//! ## Example
//!
//! ```
//! use esp32_rust_lvgl_bridge_core::{ParamId, UiAlign, WidgetDescriptor};
//! use esp32_rust_lvgl_bridge_harness::HeadlessHarness;
//!
//! const PARAM_POWER: ParamId = ParamId::new(101);
//!
//! let mut harness = HeadlessHarness::new(480, 480);
//! harness.build_schema(&[WidgetDescriptor::switch(
//!     PARAM_POWER, UiAlign::Center, 0, 0, c"On", c"Off",
//! )]);
//! harness.step(50);
//! assert!(harness.click(PARAM_POWER));
//! let cmd = harness.recv_command().expect("click emits a command");
//! assert_eq!(cmd.param_id, PARAM_POWER);
//! ```

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

    fn harness_widget_exists(param_id: u32) -> bool;
    fn harness_tabview_set_active(tv: *mut c_void, tab_idx: u32) -> bool;
    fn harness_tabview_get_active(tv: *mut c_void) -> u32;
    fn harness_label_get_text(obj: *mut c_void) -> *const c_char;
    fn harness_textarea_get_text(obj: *mut c_void) -> *const c_char;
    fn harness_textarea_set_text(obj: *mut c_void, text: *const c_char) -> bool;
    fn harness_led_get_brightness(obj: *mut c_void) -> u8;
    fn harness_click_buttonmatrix_by_text(bm: *mut c_void, text: *const c_char) -> bool;
    fn harness_click_buttonmatrix_by_id(bm: *mut c_void, btn_id: u32) -> bool;
    fn harness_clean_screen();
    fn harness_widget_is_hidden(obj: *mut c_void) -> bool;
    fn harness_widget_is_disabled(obj: *mut c_void) -> bool;
    fn harness_widget_is_clickable(obj: *mut c_void) -> bool;
    fn harness_widget_get_flex_flow(obj: *mut c_void) -> i32;
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
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return false;
        }
        unsafe { harness_click_widget(widget) };
        true
    }

    /// Simulate dragging a slider bound to `param_id` to `target_val`.
    pub fn drag_slider(&mut self, param_id: ParamId, target_val: i32) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
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
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_slider_get_value(widget) })
        }
    }

    /// Query the checked/toggled state of a switch or checkbox (LV_STATE_CHECKED = 1).
    pub fn get_checked_state(&self, param_id: ParamId) -> Option<bool> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_obj_has_state(widget, 0x0001) })
        }
    }

    /// Query the current selected index of a dropdown widget.
    pub fn get_dropdown_selected(&self, param_id: ParamId) -> Option<u32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_dropdown_get_selected(widget) })
        }
    }

    /// Query the current selected index of a roller widget.
    pub fn get_roller_selected(&self, param_id: ParamId) -> Option<u32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_roller_get_selected(widget) })
        }
    }

    /// Query the current value of an arc widget.
    pub fn get_arc_value(&self, param_id: ParamId) -> Option<i32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_arc_get_value(widget) })
        }
    }

    /// Query the current value of a spinbox widget.
    pub fn get_spinbox_value(&self, param_id: ParamId) -> Option<i32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
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

    /// Check whether a widget bound to `param_id` exists in the active scene graph.
    pub fn widget_exists(&self, param_id: ParamId) -> bool {
        unsafe { harness_widget_exists(param_id.as_u32()) }
    }

    /// Clear the active screen and re-initialize the UI engine.
    pub fn clean_screen(&mut self) {
        unsafe { harness_clean_screen() };
    }

    /// Switch active tab of a TabView widget.
    pub fn set_active_tab(&mut self, param_id: ParamId, tab_idx: u32) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return false;
        }
        unsafe { harness_tabview_set_active(widget, tab_idx) }
    }

    /// Get current active tab index of a TabView widget.
    pub fn get_active_tab(&self, param_id: ParamId) -> Option<u32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { harness_tabview_get_active(widget) })
        }
    }

    /// Query the current value of a bar widget.
    pub fn get_bar_value(&self, param_id: ParamId) -> Option<i32> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { lv_bar_get_value(widget) })
        }
    }

    /// Query the text content of a label widget.
    pub fn get_label_text(&self, param_id: ParamId) -> Option<String> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return None;
        }
        let ptr = unsafe { harness_label_get_text(widget) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
        }
    }

    /// Query the current text of a text area or text input widget.
    pub fn get_textarea_text(&self, param_id: ParamId) -> Option<String> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return None;
        }
        let ptr = unsafe { harness_textarea_get_text(widget) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
        }
    }

    /// Set text in a text area or text input widget.
    pub fn set_textarea_text(&mut self, param_id: ParamId, text: &str) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return false;
        }
        let c_text = match CString::new(text) {
            Ok(c) => c,
            Err(_) => return false,
        };
        unsafe { harness_textarea_set_text(widget, c_text.as_ptr()) }
    }

    /// Click a button matrix button by its label text (e.g. "5" on keypad).
    pub fn click_button_matrix_text(&mut self, param_id: ParamId, text: &str) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return false;
        }
        let c_text = match CString::new(text) {
            Ok(c) => c,
            Err(_) => return false,
        };
        unsafe { harness_click_buttonmatrix_by_text(widget, c_text.as_ptr()) }
    }

    /// Click a button matrix button by its button index.
    pub fn click_button_matrix_id(&mut self, param_id: ParamId, btn_id: u32) -> bool {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            return false;
        }
        unsafe { harness_click_buttonmatrix_by_id(widget, btn_id) }
    }

    /// Query the brightness of an LED widget (0 to 255).
    pub fn get_led_brightness(&self, param_id: ParamId) -> Option<u8> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { harness_led_get_brightness(widget) })
        }
    }

    /// Check if a widget is currently hidden (`LV_OBJ_FLAG_HIDDEN`).
    pub fn is_widget_hidden(&self, param_id: ParamId) -> Option<bool> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { harness_widget_is_hidden(widget) })
        }
    }

    /// Check if a widget is currently disabled (`LV_STATE_DISABLED`).
    pub fn is_widget_disabled(&self, param_id: ParamId) -> Option<bool> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { harness_widget_is_disabled(widget) })
        }
    }

    /// Check if a widget is currently clickable (`LV_OBJ_FLAG_CLICKABLE`).
    pub fn is_widget_clickable(&self, param_id: ParamId) -> Option<bool> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            Some(unsafe { harness_widget_is_clickable(widget) })
        }
    }

    /// Query the flex layout flow of a widget.
    pub fn get_widget_flex_flow(&self, param_id: ParamId) -> Option<esp32_rust_lvgl_bridge_core::abi::LayoutFlow> {
        let widget = unsafe { ui_engine_get_widget_by_param(param_id.as_u32()) };
        if widget.is_null() {
            None
        } else {
            use esp32_rust_lvgl_bridge_core::abi::LayoutFlow;
            let raw = unsafe { harness_widget_get_flex_flow(widget) };
            match raw {
                1 => Some(LayoutFlow::Row),
                2 => Some(LayoutFlow::Column),
                3 => Some(LayoutFlow::RowWrap),
                4 => Some(LayoutFlow::RowReverse),
                5 => Some(LayoutFlow::ColumnWrap),
                6 => Some(LayoutFlow::ColumnReverse),
                _ => Some(LayoutFlow::None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use esp32_rust_lvgl_bridge_core::abi::{ParamVal, UiAlign};
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_harness_resolution_and_backlight() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut harness = HeadlessHarness::new(320, 240);
        assert!(harness.backlight_is_on());

        harness.set_resolution(800, 480);
        harness.step(10);
    }

    #[test]
    fn test_widget_queries_for_nonexistent_param() {
        let _guard = TEST_LOCK.lock().unwrap();
        let harness = HeadlessHarness::new(480, 480);
        let unknown = ParamId::new(9999);

        assert_eq!(harness.get_slider_value(unknown), None);
        assert_eq!(harness.get_checked_state(unknown), None);
        assert_eq!(harness.get_dropdown_selected(unknown), None);
        assert_eq!(harness.get_roller_selected(unknown), None);
        assert_eq!(harness.get_arc_value(unknown), None);
        assert_eq!(harness.get_spinbox_value(unknown), None);
    }

    #[test]
    fn test_click_and_drag_nonexistent_widget() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut harness = HeadlessHarness::new(480, 480);
        let unknown = ParamId::new(8888);

        assert!(!harness.click(unknown));
        assert!(!harness.drag_slider(unknown, 50));
    }

    #[test]
    fn test_command_transport_queue() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut harness = HeadlessHarness::new(480, 480);

        // Queue should initially be empty
        assert!(harness.recv_command().is_none());

        // Inject command from UI
        let cmd = UiCommand {
            param_id: ParamId::new(42),
            seq_id: 1,
            value: ParamVal::from_i32(100),
        };
        let ok = unsafe { bridge_send_command_from_ui(&cmd) };
        assert!(ok);

        let received = harness.recv_command();
        assert!(received.is_some());
        let rec_cmd = received.unwrap();
        assert_eq!(rec_cmd.param_id, ParamId::new(42));
        assert_eq!(rec_cmd.seq_id, 1);
        assert_eq!(unsafe { rec_cmd.value.i32_val }, 100);

        // Queue should now be empty again
        assert!(harness.recv_command().is_none());
    }

    #[test]
    fn test_save_png_invalid_path() {
        let _guard = TEST_LOCK.lock().unwrap();
        let harness = HeadlessHarness::new(100, 100);
        let res = harness.save_png("/nonexistent_directory_999999/test.png");
        assert!(res.is_err());
    }

    #[test]
    fn test_arc_and_checkbox_queries() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut harness = HeadlessHarness::new(480, 480);

        const P_ARC: ParamId = ParamId::new(201);
        const P_CB: ParamId = ParamId::new(202);

        let schema = &[
            WidgetDescriptor::arc(
                P_ARC,
                UiAlign::Center,
                0,
                0,
                0,
                100,
                42,
                0,
                c"%ld",
                c"%ld",
            ),
            WidgetDescriptor::checkbox(
                P_CB,
                UiAlign::TopLeft,
                10,
                10,
                c"Check",
                true,
            ),
        ];

        harness.build_schema(schema);
        harness.step(20);

        assert_eq!(harness.get_arc_value(P_ARC), Some(42));
        assert_eq!(harness.get_checked_state(P_CB), Some(true));
    }
}
