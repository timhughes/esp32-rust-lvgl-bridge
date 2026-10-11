# Rust & C FFI Contract Reference

The boundary between C (ESP-IDF / LVGL) and Rust (`no_std`) relies strictly on Plain Old Data (POD) structures passed by value across FreeRTOS queues. Zero dynamic heap pointers or raw widget handles cross the boundary.

---

## Parameter IDs (`param_id_t` / `ParamId`)

Each interactive or observable parameter in the system has a unique, deterministic ID:

```c
typedef enum {
    PARAM_NONE           = 0,

    // Tier 1: Scalar Controls & Indicators
    PARAM_SETPOINT_HZ    = 1,  // Slider: i32_val setpoint
    PARAM_SYSTEM_ENABLE  = 2,  // Switch: bool_val enable/run state
    PARAM_TRIGGER_RESET  = 3,  // Push Button: momentary command trigger
    PARAM_CYCLE_COUNT    = 4,  // Readout: u32_val telemetry counter
    PARAM_ARC_KNOB       = 5,  // Arc: i32_val knob setpoint
    PARAM_PROGRESS_BAR   = 6,  // Bar: i32_val level indicator
    PARAM_AUX_CHECKBOX   = 7,  // Checkbox: bool_val check state
    PARAM_STATUS_LED     = 8,  // LED: bool_val status
    PARAM_BUSY_SPINNER   = 9,  // Spinner: bool_val visibility
    PARAM_MODE_DROPDOWN  = 10, // Dropdown: i32_val option index
    PARAM_SPEED_ROLLER   = 11, // Roller: i32_val option index
    PARAM_PRECISION_STEP = 12, // Spinbox: i32_val step value

    // Tier 2: Text Inputs, Keypads, Modals & Gauges
    PARAM_TEXT_INPUT     = 13, // TextArea: str_val buffer
    PARAM_KEYBOARD       = 14, // Keyboard: i32_val action (1=ready, 2=cancel)
    PARAM_KEYPAD_MATRIX  = 15, // ButtonMatrix: i32_val index, str_val key
    PARAM_ALERT_MODAL    = 16, // MsgBox: i32_val btn, bool_val open/close
    PARAM_GAUGE_SCALE    = 17, // Scale: i32_val needle value

    // Tier 3: Containers, Navigation & Data Streams
    PARAM_NAV_TAB        = 18, // TabView: i32_val active tab index
    PARAM_NAV_TILE       = 19, // TileView: i32_val active tile index
    PARAM_MENU_NAV       = 20, // Menu: i32_val selected item
    PARAM_ITEM_LIST      = 21, // List: i32_val selected item
    PARAM_TELEMETRY_CHART= 22, // Chart: i32_val streaming sample
    PARAM_DATA_TABLE     = 23, // Table: i32_val / str_val
    PARAM_CALENDAR_DATE  = 24, // Calendar: u32_val encoded YYYYMMDD
    PARAM_CANVAS_DRAW    = 25, // Canvas: custom pixel draw trigger
    PARAM_TEXT_AREA      = 26, // TextArea: str_val multiline buffer
} param_id_t;
```

In Rust (`crates/esp32-rust-lvgl-bridge-core/src/abi.rs`), `ParamId` is defined as an open `#[repr(transparent)]` `u32` newtype:

```rust
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ParamId(pub u32);

impl ParamId {
    pub const None: Self = Self(0);
    pub const SetpointHz: Self = Self(1);
    // ... predefined framework constants ...

    pub const fn new(id: u32) -> Self { Self(id) }
    pub const fn as_u32(self) -> u32 { self.0 }
}
```

This allows custom user applications to define domain-specific parameters without modifying the core bridge library:
```rust
pub const TARGET_TEMPERATURE: ParamId = ParamId::new(101);
pub const FAN_SPEED: ParamId = ParamId::new(102);
```

---

## Binary Values (`param_val_t` / `ParamVal`)

Values are packed into a fixed-size, 32-byte union. The inclusion of `char str_val[32]` allows text inputs and button matrix keys to be transmitted across FreeRTOS queues without dynamic heap allocations or dangling pointer risks:

```c
typedef union {
    int32_t  i32_val;
    uint32_t u32_val;
    float    f32_val;
    bool     bool_val;
    char     str_val[32];      // Zero-allocation fixed string buffer (NUL-terminated)
} param_val_t;
```

In Rust (`crates/esp32-rust-lvgl-bridge-core/src/abi.rs`):
```rust
#[repr(C)]
#[derive(Copy, Clone)]
pub union ParamVal {
    pub i32_val: i32,
    pub u32_val: u32,
    pub f32_val: f32,
    pub bool_val: bool,
    pub str_val: [u8; 32],
}
```

---

## Message Structs

```c
// UI -> Rust Worker (Command Queue)
typedef struct {
    param_id_t  param_id;
    uint32_t    seq_id;         // Monotonic ticket for UI arbitration
    param_val_t value;
} ui_command_t;

// Rust Worker -> UI (Telemetry Queue)
typedef struct {
    param_id_t  param_id;
    uint32_t    ack_seq_id;     // Mirrored seq_id from acknowledged command
    param_val_t value;
} device_telemetry_t;
```

---

## Declarative Widget Types (`widget_type_t` / `WidgetType`)

```c
typedef enum {
    WIDGET_TYPE_SWITCH        = 1,
    WIDGET_TYPE_SLIDER        = 2,
    WIDGET_TYPE_BUTTON        = 3,
    WIDGET_TYPE_LABEL         = 4,
    WIDGET_TYPE_ARC           = 5,
    WIDGET_TYPE_BAR           = 6,
    WIDGET_TYPE_CHECKBOX      = 7,
    WIDGET_TYPE_LED           = 8,
    WIDGET_TYPE_SPINNER       = 9,
    WIDGET_TYPE_DROPDOWN      = 10,
    WIDGET_TYPE_ROLLER        = 11,
    WIDGET_TYPE_SPINBOX       = 12,
    // Tier 2
    WIDGET_TYPE_TEXTAREA      = 13,
    WIDGET_TYPE_KEYBOARD      = 14,
    WIDGET_TYPE_BUTTON_MATRIX = 15,
    WIDGET_TYPE_MSGBOX        = 16,
    WIDGET_TYPE_SCALE         = 17,
    // Tier 3
    WIDGET_TYPE_TABVIEW       = 18,
    WIDGET_TYPE_TILEVIEW      = 19,
    WIDGET_TYPE_MENU          = 20,
    WIDGET_TYPE_LIST          = 21,
    WIDGET_TYPE_CHART         = 22,
    WIDGET_TYPE_TABLE         = 23,
    WIDGET_TYPE_CALENDAR      = 24,
    WIDGET_TYPE_CANVAS        = 25,
    // Phase 1: Visuals & IoT Core
    WIDGET_TYPE_IMAGE         = 26,
    WIDGET_TYPE_LINE          = 27,
    WIDGET_TYPE_ARC_LABEL     = 28,
    WIDGET_TYPE_QR_CODE       = 29,
    // Phase 2: Complex UI & Rich Media
    WIDGET_TYPE_WIN           = 30,
    WIDGET_TYPE_IMAGEBUTTON   = 31,
    WIDGET_TYPE_SPAN_GROUP    = 32,
    WIDGET_TYPE_ANIMIMAGE     = 33,
    WIDGET_TYPE_BARCODE       = 34,
} widget_type_t;
```

---

## Widget Descriptor (`widget_descriptor_t` / `WidgetDescriptor`)

Static data table defining UI layout, hierarchy, and binding behavior:

```c
typedef struct {
    widget_type_t widget_type;
    param_id_t    param_id;
    ui_align_t    align;
    int32_t       offset_x;
    int32_t       offset_y;
    int32_t       min_val;
    int32_t       max_val;
    int32_t       initial_val;
    uint32_t      holdoff_ms;
    const char   *label_primary;
    const char   *label_secondary;
    param_id_t    parent_param_id; // Parameter ID of container (e.g. TabView)
    int32_t       width;           // Explicit width (0 = default/auto)
    int32_t       height;          // Explicit height (0 = default/auto)
    uint32_t      tab_index;       // Target tab index inside parent TabView
} widget_descriptor_t;
```

---

## Exported Rust C Entry Points

Application staticlibs (such as `everything-app` and `minimal-app`) export the following functions with C linkage (`extern "C"`):

| Function | Signature | Description |
|---|---|---|
| `rust_worker_init` | `void rust_worker_init(void)` | Initializes the static `SystemEngine` state machine. |
| `rust_worker_step` | `void rust_worker_step(void)` | Consumes incoming commands, steps simulation, and emits telemetry. |
| `rust_get_minimal_schema` | `const widget_descriptor_t *rust_get_minimal_schema(size_t *count)` | Returns minimal starter schema (`examples/minimal`). |
| `rust_get_everything_schema` | `const widget_descriptor_t *rust_get_everything_schema(size_t *count)` | Returns the complete 25-widget 6-tab schema (`examples/everything`). |

---

## Transport Hooks Implemented by C Runtime

The Rust domain logic calls these functions to communicate with the FreeRTOS queues:

* `bool bridge_recv_command(ui_command_t *out_cmd, uint32_t timeout_ms)`: Pops an incoming command sent from LVGL touch events.
* `bool bridge_send_telemetry(const device_telemetry_t *telem)`: Pushes an updated telemetry packet into the queue for LVGL to consume.

---

## Phase 3: Declarative Layouts & Runtime Widget Flags

- `WidgetDescriptor::with_layout(LayoutFlow)` sets a flex flow (`Row`, `Column`, `RowWrap`, `RowReverse`, `ColumnWrap`, `ColumnReverse`) on the created widget (`layout_flow` field in `widget_descriptor_t`).
- Runtime state changes use telemetry: `DeviceTelemetry::widget_flag_update(target, flags)` sends `PARAM_WIDGET_FLAGS` with `ack_seq_id = target param` and `u32_val` flags from `WidgetFlagOp` (`Hidden`, `Visible`, `Disabled`, `Enabled`, `Clickable`, `Unclickable`).
