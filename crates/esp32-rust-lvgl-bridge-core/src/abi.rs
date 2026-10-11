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


