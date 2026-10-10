#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParamId {
    SetpointHz = 1,
    SystemEnable = 2,
    TriggerReset = 3,
    CycleCount = 4,
    ArcKnob = 5,
    ProgressBar = 6,
    AuxCheckbox = 7,
    StatusLed = 8,
    BusySpinner = 9,
    ModeDropdown = 10,
    SpeedRoller = 11,
    PrecisionStep = 12,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ParamVal {
    pub i32_val: i32,
    pub u32_val: u32,
    pub f32_val: f32,
    pub bool_val: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UiCommand {
    pub param_id: ParamId,
    pub seq_id: u32,
    pub value: ParamVal,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DeviceTelemetry {
    pub param_id: ParamId,
    pub ack_seq_id: u32,
    pub value: ParamVal,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WidgetType {
    Switch = 1,
    Slider = 2,
    Button = 3,
    Label = 4,
    Arc = 5,
    Bar = 6,
    Checkbox = 7,
    Led = 8,
    Spinner = 9,
    Dropdown = 10,
    Roller = 11,
    Spinbox = 12,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UiAlign {
    TopLeft = 1,
    TopMid = 2,
    TopRight = 3,
    Center = 4,
    BottomLeft = 5,
    BottomMid = 6,
    BottomRight = 7,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WidgetDescriptor {
    pub widget_type: WidgetType,
    pub param_id: ParamId,
    pub align: UiAlign,
    pub offset_x: i32,
    pub offset_y: i32,
    pub min_val: i32,
    pub max_val: i32,
    pub initial_val: i32,
    pub holdoff_ms: u32,
    pub label_primary: *const core::ffi::c_char,
    pub label_secondary: *const core::ffi::c_char,
}

unsafe impl Sync for WidgetDescriptor {}
unsafe impl Send for WidgetDescriptor {}

impl WidgetDescriptor {
    pub const fn switch(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        label_on: &'static core::ffi::CStr,
        label_off: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Switch,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 1,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: label_on.as_ptr(),
            label_secondary: label_off.as_ptr(),
        }
    }

    pub const fn slider(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        min: i32,
        max: i32,
        initial: i32,
        holdoff_ms: u32,
        fmt_setting: &'static core::ffi::CStr,
        fmt_actual: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Slider,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: min,
            max_val: max,
            initial_val: initial,
            holdoff_ms,
            label_primary: fmt_setting.as_ptr(),
            label_secondary: fmt_actual.as_ptr(),
        }
    }

    pub const fn button(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        text: &'static core::ffi::CStr,
        cmd_val: u32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Button,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: cmd_val as i32,
            holdoff_ms: 0,
            label_primary: text.as_ptr(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn label(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        format: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Label,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: format.as_ptr(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn arc(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        min: i32,
        max: i32,
        initial: i32,
        holdoff_ms: u32,
        fmt_setting: &'static core::ffi::CStr,
        fmt_actual: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Arc,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: min,
            max_val: max,
            initial_val: initial,
            holdoff_ms,
            label_primary: fmt_setting.as_ptr(),
            label_secondary: fmt_actual.as_ptr(),
        }
    }

    pub const fn bar(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        min: i32,
        max: i32,
        initial: i32,
        format: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Bar,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: min,
            max_val: max,
            initial_val: initial,
            holdoff_ms: 0,
            label_primary: format.as_ptr(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn checkbox(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        text: &'static core::ffi::CStr,
        initial_checked: bool,
    ) -> Self {
        Self {
            widget_type: WidgetType::Checkbox,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 1,
            initial_val: initial_checked as i32,
            holdoff_ms: 0,
            label_primary: text.as_ptr(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn led(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        initial_on: bool,
    ) -> Self {
        Self {
            widget_type: WidgetType::Led,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 255,
            initial_val: initial_on as i32,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn spinner(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        initial_visible: bool,
    ) -> Self {
        Self {
            widget_type: WidgetType::Spinner,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 1,
            initial_val: initial_visible as i32,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn dropdown(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        options: &'static core::ffi::CStr,
        initial_idx: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Dropdown,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 100,
            initial_val: initial_idx,
            holdoff_ms: 0,
            label_primary: options.as_ptr(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn roller(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        options: &'static core::ffi::CStr,
        initial_idx: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Roller,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 100,
            initial_val: initial_idx,
            holdoff_ms: 0,
            label_primary: options.as_ptr(),
            label_secondary: core::ptr::null(),
        }
    }

    pub const fn spinbox(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        min: i32,
        max: i32,
        initial: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Spinbox,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: min,
            max_val: max,
            initial_val: initial,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
        }
    }
}
