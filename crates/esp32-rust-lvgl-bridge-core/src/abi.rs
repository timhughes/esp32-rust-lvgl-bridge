//! # Bridge Core ABI: C-Compatible Foreign Function Interface (FFI) Contracts
//!
//! Defines the Plain Old Data (POD) structures, unions, enumerations, and
//! declarative widget constructors shared between ESP-IDF FreeRTOS C tasks and
//! the Rust domain logic.

/// Parameter identifier bridging Rust domain logic and C LVGL widget bindings.
///
/// Encapsulated as an open `u32` newtype with C ABI transparency (`#[repr(transparent)]`),
/// allowing applications to declare custom parameter IDs while retaining predefined constants
/// for standard framework widgets.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ParamId(pub u32);

#[allow(non_upper_case_globals)]
impl ParamId {
    pub const None: Self = Self(0);
    pub const SetpointHz: Self = Self(1);
    pub const SystemEnable: Self = Self(2);
    pub const TriggerReset: Self = Self(3);
    pub const CycleCount: Self = Self(4);
    pub const ArcKnob: Self = Self(5);
    pub const ProgressBar: Self = Self(6);
    pub const AuxCheckbox: Self = Self(7);
    pub const StatusLed: Self = Self(8);
    pub const BusySpinner: Self = Self(9);
    pub const ModeDropdown: Self = Self(10);
    pub const SpeedRoller: Self = Self(11);
    pub const PrecisionStep: Self = Self(12);
    // Tier 2: Text, Keypads, Modals & Gauges
    pub const TextInput: Self = Self(13);
    pub const Keyboard: Self = Self(14);
    pub const KeypadMatrix: Self = Self(15);
    pub const AlertModal: Self = Self(16);
    pub const GaugeScale: Self = Self(17);
    // Tier 3: Containers, Navigation & Data Streams
    pub const NavTab: Self = Self(18);
    pub const NavTile: Self = Self(19);
    pub const MenuNav: Self = Self(20);
    pub const ItemList: Self = Self(21);
    pub const TelemetryChart: Self = Self(22);
    pub const DataTable: Self = Self(23);
    pub const CalendarDate: Self = Self(24);
    pub const CanvasDraw: Self = Self(25);
    pub const TextArea: Self = Self(26);

    /// Creates a custom application parameter ID.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the underlying raw 32-bit identifier.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl From<u32> for ParamId {
    #[inline]
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<ParamId> for u32 {
    #[inline]
    fn from(param: ParamId) -> Self {
        param.0
    }
}


#[repr(C)]
#[derive(Copy, Clone)]
pub union ParamVal {
    pub i32_val: i32,
    pub u32_val: u32,
    pub f32_val: f32,
    pub bool_val: bool,
    pub str_val: [u8; 32],
}

impl ParamVal {
    pub const fn from_i32(v: i32) -> Self {
        Self { i32_val: v }
    }
    pub const fn from_u32(v: u32) -> Self {
        Self { u32_val: v }
    }
    pub const fn from_bool(v: bool) -> Self {
        Self { bool_val: v }
    }
    pub const fn from_str(s: &[u8]) -> Self {
        let mut buf = [0u8; 32];
        let mut i = 0;
        while i < s.len() && i < 31 {
            buf[i] = s[i];
            i += 1;
        }
        buf[i] = 0;
        Self { str_val: buf }
    }
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
    // Tier 2
    TextArea = 13,
    Keyboard = 14,
    ButtonMatrix = 15,
    MsgBox = 16,
    Scale = 17,
    // Tier 3
    TabView = 18,
    TileView = 19,
    Menu = 20,
    List = 21,
    Chart = 22,
    Table = 23,
    Calendar = 24,
    Canvas = 25,
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
    pub parent_param_id: ParamId,
    pub width: i32,
    pub height: i32,
    pub tab_index: u32,
}

unsafe impl Sync for WidgetDescriptor {}
unsafe impl Send for WidgetDescriptor {}

impl WidgetDescriptor {
    // -------------------------------------------------------------------------
    // Tier 1 Constructors
    // -------------------------------------------------------------------------

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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
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
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
        }
    }

    // -------------------------------------------------------------------------
    // Tier 2 Constructors: Text Inputs, Keypads, Modals & Gauges
    // -------------------------------------------------------------------------

    pub const fn text_area(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        placeholder: &'static core::ffi::CStr,
        default_text: Option<&'static core::ffi::CStr>,
        one_line: bool,
        max_len: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::TextArea,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: if one_line { 1 } else { 0 },
            max_val: max_len,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: placeholder.as_ptr(),
            label_secondary: match default_text {
                Some(t) => t.as_ptr(),
                None => core::ptr::null(),
            },
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    pub const fn text_input(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        placeholder: &'static core::ffi::CStr,
        default_text: Option<&'static core::ffi::CStr>,
        max_len: i32,
    ) -> Self {
        Self::text_area(
            param_id,
            align,
            offset_x,
            offset_y,
            width,
            height,
            placeholder,
            default_text,
            true,
            max_len,
        )
    }

    pub const fn keyboard(
        param_id: ParamId,
        target_textarea: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Keyboard,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: target_textarea.0 as i32,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    pub const fn button_matrix(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        map_str: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::ButtonMatrix,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: map_str.as_ptr(),
            label_secondary: core::ptr::null(),
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    pub const fn msgbox(
        param_id: ParamId,
        title: &'static core::ffi::CStr,
        message: &'static core::ffi::CStr,
        initial_visible: bool,
    ) -> Self {
        Self {
            widget_type: WidgetType::MsgBox,
            param_id,
            align: UiAlign::Center,
            offset_x: 0,
            offset_y: 0,
            min_val: 0,
            max_val: 0,
            initial_val: initial_visible as i32,
            holdoff_ms: 0,
            label_primary: title.as_ptr(),
            label_secondary: message.as_ptr(),
            parent_param_id: ParamId::None,
            width: 0,
            height: 0,
            tab_index: 0,
        }
    }

    pub const fn scale(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        min: i32,
        max: i32,
        initial: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Scale,
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
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    // -------------------------------------------------------------------------
    // Tier 3 Constructors: Containers, Navigation & Data Streaming
    // -------------------------------------------------------------------------

    pub const fn tabview(
        param_id: ParamId,
        tab_names: &'static core::ffi::CStr,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::TabView,
            param_id,
            align: UiAlign::Center,
            offset_x: 0,
            offset_y: 0,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: tab_names.as_ptr(),
            label_secondary: core::ptr::null(),
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    pub const fn tileview(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::TileView,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    pub const fn menu(
        param_id: ParamId,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        title: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Menu,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: title.as_ptr(),
            label_secondary: core::ptr::null(),
            parent_param_id: ParamId::None,
            width,
            height,
            tab_index: 0,
        }
    }

    pub const fn list(
        param_id: ParamId,
        parent_id: ParamId,
        tab_index: u32,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        items: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::List,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: items.as_ptr(),
            label_secondary: core::ptr::null(),
            parent_param_id: parent_id,
            width,
            height,
            tab_index,
        }
    }

    pub const fn chart(
        param_id: ParamId,
        parent_id: ParamId,
        tab_index: u32,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        min: i32,
        max: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Chart,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: min,
            max_val: max,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
            parent_param_id: parent_id,
            width,
            height,
            tab_index,
        }
    }

    pub const fn table(
        param_id: ParamId,
        parent_id: ParamId,
        tab_index: u32,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
        rows_cols: i32, // encoded (rows << 16) | cols
        initial_csv: &'static core::ffi::CStr,
    ) -> Self {
        Self {
            widget_type: WidgetType::Table,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: rows_cols,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: initial_csv.as_ptr(),
            label_secondary: core::ptr::null(),
            parent_param_id: parent_id,
            width,
            height,
            tab_index,
        }
    }

    pub const fn calendar(
        param_id: ParamId,
        parent_id: ParamId,
        tab_index: u32,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Calendar,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
            parent_param_id: parent_id,
            width,
            height,
            tab_index,
        }
    }

    pub const fn canvas(
        param_id: ParamId,
        parent_id: ParamId,
        tab_index: u32,
        align: UiAlign,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            widget_type: WidgetType::Canvas,
            param_id,
            align,
            offset_x,
            offset_y,
            min_val: 0,
            max_val: 0,
            initial_val: 0,
            holdoff_ms: 0,
            label_primary: core::ptr::null(),
            label_secondary: core::ptr::null(),
            parent_param_id: parent_id,
            width,
            height,
            tab_index,
        }
    }

    pub const fn with_parent(mut self, parent_param_id: ParamId, tab_index: u32) -> Self {
        self.parent_param_id = parent_param_id;
        self.tab_index = tab_index;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::ffi::CStr;

    #[test]
    fn test_param_id_conversions_and_constants() {
        let p = ParamId::new(42);
        assert_eq!(p.as_u32(), 42);
        assert_eq!(ParamId::from(100u32), ParamId(100));
        assert_eq!(u32::from(ParamId(200)), 200u32);

        assert_eq!(ParamId::None, ParamId(0));
        assert_eq!(ParamId::SetpointHz, ParamId(1));
        assert_eq!(ParamId::SystemEnable, ParamId(2));
        assert_eq!(ParamId::TriggerReset, ParamId(3));
        assert_eq!(ParamId::CycleCount, ParamId(4));
        assert_eq!(ParamId::ArcKnob, ParamId(5));
        assert_eq!(ParamId::ProgressBar, ParamId(6));
        assert_eq!(ParamId::AuxCheckbox, ParamId(7));
        assert_eq!(ParamId::StatusLed, ParamId(8));
        assert_eq!(ParamId::BusySpinner, ParamId(9));
        assert_eq!(ParamId::ModeDropdown, ParamId(10));
        assert_eq!(ParamId::SpeedRoller, ParamId(11));
        assert_eq!(ParamId::PrecisionStep, ParamId(12));
        assert_eq!(ParamId::TextInput, ParamId(13));
        assert_eq!(ParamId::Keyboard, ParamId(14));
        assert_eq!(ParamId::KeypadMatrix, ParamId(15));
        assert_eq!(ParamId::AlertModal, ParamId(16));
        assert_eq!(ParamId::GaugeScale, ParamId(17));
        assert_eq!(ParamId::NavTab, ParamId(18));
        assert_eq!(ParamId::NavTile, ParamId(19));
        assert_eq!(ParamId::MenuNav, ParamId(20));
        assert_eq!(ParamId::ItemList, ParamId(21));
        assert_eq!(ParamId::TelemetryChart, ParamId(22));
        assert_eq!(ParamId::DataTable, ParamId(23));
        assert_eq!(ParamId::CalendarDate, ParamId(24));
        assert_eq!(ParamId::CanvasDraw, ParamId(25));
        assert_eq!(ParamId::TextArea, ParamId(26));

        // Test Hash and Eq in a HashSet
        let mut set = HashSet::new();
        set.insert(ParamId(1));
        set.insert(ParamId(2));
        set.insert(ParamId(1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_param_val_primitives() {
        let v_i32 = ParamVal::from_i32(-999);
        assert_eq!(unsafe { v_i32.i32_val }, -999);

        let v_u32 = ParamVal::from_u32(123456);
        assert_eq!(unsafe { v_u32.u32_val }, 123456);

        let v_true = ParamVal::from_bool(true);
        assert_eq!(unsafe { v_true.bool_val }, true);

        let v_false = ParamVal::from_bool(false);
        assert_eq!(unsafe { v_false.bool_val }, false);
    }

    #[test]
    fn test_param_val_string_and_truncation() {
        let short = ParamVal::from_str(b"ESP32-S3");
        let bytes = unsafe { &short.str_val };
        assert_eq!(&bytes[..8], b"ESP32-S3");
        assert_eq!(bytes[8], 0);

        // Exactly 31 characters
        let exact31 = b"1234567890123456789012345678901";
        let val31 = ParamVal::from_str(exact31);
        let bytes31 = unsafe { &val31.str_val };
        assert_eq!(&bytes31[..31], exact31);
        assert_eq!(bytes31[31], 0);

        // Longer than 31 characters (must truncate to 31 + null terminator)
        let long = b"This is a very long string that definitely exceeds the 32 byte buffer limit!";
        let val_long = ParamVal::from_str(long);
        let bytes_long = unsafe { &val_long.str_val };
        assert_eq!(&bytes_long[..31], &long[..31]);
        assert_eq!(bytes_long[31], 0);
    }

    #[test]
    fn test_ui_command_and_telemetry_layout() {
        let cmd = UiCommand {
            param_id: ParamId(42),
            seq_id: 101,
            value: ParamVal::from_i32(50),
        };
        assert_eq!(cmd.param_id, ParamId(42));
        assert_eq!(cmd.seq_id, 101);
        assert_eq!(unsafe { cmd.value.i32_val }, 50);

        let telem = DeviceTelemetry {
            param_id: ParamId(42),
            ack_seq_id: 101,
            value: ParamVal::from_bool(true),
        };
        assert_eq!(telem.param_id, ParamId(42));
        assert_eq!(telem.ack_seq_id, 101);
        assert_eq!(unsafe { telem.value.bool_val }, true);
    }

    #[test]
    fn test_tier1_widget_constructors() {
        let sw = WidgetDescriptor::switch(
            ParamId(1),
            UiAlign::TopLeft,
            10,
            20,
            c"ON",
            c"OFF",
        );
        assert_eq!(sw.widget_type, WidgetType::Switch);
        assert_eq!(sw.param_id, ParamId(1));
        assert_eq!(sw.align, UiAlign::TopLeft);
        assert_eq!(sw.offset_x, 10);
        assert_eq!(sw.offset_y, 20);
        assert_eq!(sw.min_val, 0);
        assert_eq!(sw.max_val, 1);
        assert_eq!(unsafe { CStr::from_ptr(sw.label_primary).to_str().unwrap() }, "ON");
        assert_eq!(unsafe { CStr::from_ptr(sw.label_secondary).to_str().unwrap() }, "OFF");

        let sl = WidgetDescriptor::slider(
            ParamId(2),
            UiAlign::Center,
            0,
            0,
            10,
            100,
            50,
            300,
            c"Val: %ld",
            c"Act: %ld",
        );
        assert_eq!(sl.widget_type, WidgetType::Slider);
        assert_eq!(sl.min_val, 10);
        assert_eq!(sl.max_val, 100);
        assert_eq!(sl.initial_val, 50);
        assert_eq!(sl.holdoff_ms, 300);

        let btn = WidgetDescriptor::button(
            ParamId(3),
            UiAlign::BottomMid,
            0,
            -20,
            c"Click Me",
            1,
        );
        assert_eq!(btn.widget_type, WidgetType::Button);
        assert_eq!(btn.initial_val, 1);
        assert_eq!(unsafe { CStr::from_ptr(btn.label_primary).to_str().unwrap() }, "Click Me");
        assert!(btn.label_secondary.is_null());

        let lbl = WidgetDescriptor::label(
            ParamId(4),
            UiAlign::TopMid,
            0,
            15,
            c"Title: %s",
        );
        assert_eq!(lbl.widget_type, WidgetType::Label);

        let arc = WidgetDescriptor::arc(
            ParamId(5),
            UiAlign::Center,
            0,
            0,
            0,
            100,
            25,
            400,
            c"%ld%%",
            c"%ld%%",
        );
        assert_eq!(arc.widget_type, WidgetType::Arc);
        assert_eq!(arc.holdoff_ms, 400);

        let bar = WidgetDescriptor::bar(
            ParamId(6),
            UiAlign::BottomLeft,
            10,
            -10,
            0,
            100,
            40,
            c"%ld",
        );
        assert_eq!(bar.widget_type, WidgetType::Bar);

        let cb = WidgetDescriptor::checkbox(
            ParamId(7),
            UiAlign::TopRight,
            -10,
            10,
            c"Enable",
            true,
        );
        assert_eq!(cb.widget_type, WidgetType::Checkbox);
        assert_eq!(cb.initial_val, 1);

        let led = WidgetDescriptor::led(
            ParamId(8),
            UiAlign::TopLeft,
            5,
            5,
            false,
        );
        assert_eq!(led.widget_type, WidgetType::Led);
        assert_eq!(led.initial_val, 0);

        let spin = WidgetDescriptor::spinner(
            ParamId(9),
            UiAlign::Center,
            0,
            0,
            true,
        );
        assert_eq!(spin.widget_type, WidgetType::Spinner);
        assert_eq!(spin.initial_val, 1);

        let dd = WidgetDescriptor::dropdown(
            ParamId(10),
            UiAlign::Center,
            0,
            50,
            c"Opt1\nOpt2\nOpt3",
            1,
        );
        assert_eq!(dd.widget_type, WidgetType::Dropdown);
        assert_eq!(dd.initial_val, 1);

        let roller = WidgetDescriptor::roller(
            ParamId(11),
            UiAlign::Center,
            0,
            -50,
            c"Low\nMed\nHigh",
            2,
        );
        assert_eq!(roller.widget_type, WidgetType::Roller);
        assert_eq!(roller.initial_val, 2);

        let spinbox = WidgetDescriptor::spinbox(
            ParamId(12),
            UiAlign::BottomRight,
            -20,
            -20,
            -50,
            50,
            0,
        );
        assert_eq!(spinbox.widget_type, WidgetType::Spinbox);
        assert_eq!(spinbox.min_val, -50);
        assert_eq!(spinbox.max_val, 50);
    }

    #[test]
    fn test_tier2_and_tier3_widget_constructors() {
        let ta = WidgetDescriptor::text_area(
            ParamId(13),
            UiAlign::Center,
            0,
            0,
            200,
            100,
            c"Placeholder",
            Some(c"Default"),
            false,
            64,
        );
        assert_eq!(ta.widget_type, WidgetType::TextArea);
        assert_eq!(ta.width, 200);
        assert_eq!(ta.height, 100);
        assert_eq!(ta.min_val, 0); // multi-line
        assert_eq!(ta.max_val, 64);
        assert_eq!(unsafe { CStr::from_ptr(ta.label_primary).to_str().unwrap() }, "Placeholder");
        assert_eq!(unsafe { CStr::from_ptr(ta.label_secondary).to_str().unwrap() }, "Default");

        let kb = WidgetDescriptor::keyboard(
            ParamId(14),
            ParamId(13),
            UiAlign::BottomMid,
            0,
            0,
            320,
            120,
        );
        assert_eq!(kb.widget_type, WidgetType::Keyboard);
        assert_eq!(kb.initial_val, 13); // target textarea param_id

        let mb = WidgetDescriptor::msgbox(
            ParamId(16),
            c"Warning",
            c"High Temperature Alert",
            true,
        );
        assert_eq!(mb.widget_type, WidgetType::MsgBox);
        assert_eq!(mb.initial_val, 1);
        assert_eq!(unsafe { CStr::from_ptr(mb.label_primary).to_str().unwrap() }, "Warning");
        assert_eq!(unsafe { CStr::from_ptr(mb.label_secondary).to_str().unwrap() }, "High Temperature Alert");

        let tab = WidgetDescriptor::tabview(
            ParamId(18),
            c"Home\nSettings\nDiagnostics",
            480,
            480,
        );
        assert_eq!(tab.widget_type, WidgetType::TabView);
        assert_eq!(tab.width, 480);
        assert_eq!(tab.height, 480);

        let list = WidgetDescriptor::list(
            ParamId(21),
            ParamId(18),
            1,
            UiAlign::Center,
            0,
            0,
            200,
            200,
            c"Item1\nItem2",
        );
        assert_eq!(list.widget_type, WidgetType::List);
        assert_eq!(list.parent_param_id, ParamId(18));
        assert_eq!(list.tab_index, 1);

        let chained = WidgetDescriptor::button(
            ParamId(30),
            UiAlign::Center,
            0,
            0,
            c"Tab Button",
            0,
        ).with_parent(ParamId(18), 2);
        assert_eq!(chained.parent_param_id, ParamId(18));
        assert_eq!(chained.tab_index, 2);
    }
}


