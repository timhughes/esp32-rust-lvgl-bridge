# Declarative UI Engine

The framework replaces procedural C UI assembly with a **data-driven declarative schema** defined in pure Rust (**"C owns the display; Rust owns the system"**).

---

## Architecture Pattern Summary

| Category | Typical Widgets | FFI Data Type | Synchronization Mechanism |
|---|---|---|---|
| **Continuous Control** | `Slider`, `Arc` (Knob) | `i32_val` | 3-Stage Arbitration (`ARB_DRAGGING` $\rightarrow$ `ARB_HOLDOFF` lease + monotonic `seq_id`) |
| **Binary Toggles** | `Switch`, `Checkbox` | `bool_val` | Programmatic update suppression flag (`is_programmatic_update`) + active touch priority |
| **Discrete Selectors** | `Dropdown`, `Roller`, `Spinbox` | `i32_val` / `u32_val` | Value changed command + monotonic sequence ID |
| **Numeric Readouts** | `Label`, `Bar`, `Arc` (Gauge) | `i32_val` / `u32_val` | Direct telemetry update (`device_telemetry_t`) |
| **State Indicators** | `LED`, `Spinner` | `bool_val` | Direct state / brightness / visibility toggle |
| **Text Buffers & Keypads** | `TextArea`, `Keyboard`, `ButtonMatrix` | `char str_val[32]` | Fixed-size Plain Old Data (POD) zero-allocation buffer |
| **Overlays & Gauges** | `MsgBox`, `Scale` | `i32_val` / `bool_val` | Modal popup toggle & needle angle synchronization |
| **Hierarchical Containers** | `TabView`, `TileView`, `Menu` | Schema `parent_param_id`, `tab_index` | Rust-defined screen hierarchy & multi-page navigation |
| **Data Streams & Grids** | `Chart`, `Table`, `Calendar` | Struct / sample batches | Time-series waveforms & structured tabular views |

---

## Core Schema Struct

The schema consists of static array descriptors exposed over the C FFI boundary:

```rust
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
```

The C runtime builds and binds the entire UI scene graph dynamically from this schema table with a single call:
```c
ui_engine_build_from_schema(schema, count);
```

---

## Supported Widgets Catalog

### Tier 1: Scalar & Discrete Widgets
1. **`Switch`**: Binary toggle with active touch priority and programmatic update guards.
2. **`Slider`**: Continuous linear setpoint with 3-stage arbitration (`TRACKING` $\rightarrow$ `DRAGGING` $\rightarrow$ `HOLDOFF`).
3. **`Button`**: Momentary command trigger with monotonic sequence IDs.
4. **`Label`**: Dynamic telemetry readout with `printf`-style formatting.
5. **`Arc`**: Rotary dial knob (3-stage lease) OR circular progress gauge.
6. **`Bar`**: Linear progress bar and level indicator (`i32_val`).
7. **`Checkbox`**: Binary checkbox with text label and recursion guards.
8. **`LED`**: Virtual colored status LED (on/off, brightness 0–255).
9. **`Spinner`**: Animated activity spinner (shown when busy / hidden when idle).
10. **`Dropdown`**: Dropdown option selector (newline-separated options string).
11. **`Roller`**: Rotating cylinder wheel selector (scroll wheel).
12. **`Spinbox`**: High-precision numeric step counter with `+`/`-` increment steps.

### Tier 2: Text Inputs, Keypads & Modal Overlays
13. **`TextArea`**: Single-line or multi-line text input field with cursor management and zero-allocation `char str_val[32]` buffers.
14. **`Keyboard`**: On-screen touch QWERTY / numeric keyboard that automatically binds to and types into the focused `TextArea`.
15. **`ButtonMatrix`**: Grid of buttons packed into a single object (keypads, PIN entry, calculators, segmented controls).
16. **`MsgBox`**: Modal dialog alert overlays with title, message text, and action buttons (`OK`/`Cancel`).
17. **`Scale`**: Analog speedometer / dial gauge with tick divisions, labels, and needle indicators.

### Tier 3: Containers, Navigation & Data Streams
18. **`TabView`**: Multi-screen tabbed interface with top navigation headers (*Controls*, *Selectors*, *Inputs*, *Telemetry*, *Tools*, *Media*, *System*).
19. **`TileView`**: 2D swipeable grid container.
20. **`Menu`**: Hierarchical multi-level settings drill-down menu.
21. **`List`**: Scrollable action list with items.
22. **`Chart`**: Real-time time-series oscilloscope / waveform chart updated via streaming telemetry.
23. **`Table`**: Multi-row, multi-column tabular grid for system status, Wi-Fi scans, and diagnostic logs.
24. **`Calendar`**: Month calendar viewer and date picker.
25. **`Canvas`**: Raw pixel graphics buffer for custom drawing.

### Rich Media & Visual Assets
26. **`Image`**: Static raster graphic preview (RGB565 / PNG).
27. **`Line`**: Precise vector line segments connecting coordinate points.
28. **`ArcLabel`**: Curved circular heading label with adjustable arc angles.
29. **`QRCode`**: High-density 2D QR code generator (URLs, Wi-Fi credentials).
30. **`Win`**: Movable / dismissible window modal with title header and close button.
31. **`ImageButton`**: Graphical button with distinct normal and pressed image assets.
32. **`SpanGroup`**: Rich formatted text container with per-span coloring, fonts, and styling.
33. **`AnimImage`**: Looping sprite frame animation.
34. **`Barcode`**: 1D barcode generator for asset tracking and serial numbers.

---

## Example Rust Schema Definitions

### Modular Tab Schemas (`TAB3_TELEMETRY_SCHEMA`)

```rust
pub static TAB3_TELEMETRY_SCHEMA: &[WidgetDescriptor] = &[
    // 1. Real-Time Waveform Oscilloscope Chart
    WidgetDescriptor::chart(
        ParamId::TelemetryChart,
        ParamId::NavTab,
        3, // Mounted in Tab 3
        UiAlign::TopMid,
        0, 10,
        420, 150,
        0, 100,
    ),
    // 2. Multi-Row System Diagnostics Status Table
    WidgetDescriptor::table(
        ParamId::DataTable,
        ParamId::NavTab,
        3, // Mounted in Tab 3
        UiAlign::Center,
        0, 35,
        420, 120,
        (4 << 16) | 2, // 4 rows, 2 columns
        c"Module;Status;Core 0;OK;Core 1;OK;PSRAM;8MB",
    ),
    // 3. Hardware Peripheral List
    WidgetDescriptor::list(
        ParamId::ItemList,
        ParamId::NavTab,
        3, // Mounted in Tab 3
        UiAlign::BottomMid,
        0, -15,
        420, 75,
        c"ST7701S RGB (Active)\nGT911 I2C (Ready)\nESP32-S3 (240MHz)",
    ),
];
```

---

### Text Input, Multi-Line Text Area & Virtual Keyboard

The framework provides first-class support for both single-line form inputs and multi-line text editors paired with an on-screen virtual keyboard:

```rust
pub static INPUTS_SCHEMA: &[WidgetDescriptor] = &[
    // 1. Single-Line Text Input Field (e.g. command or Wi-Fi SSID)
    WidgetDescriptor::text_input(
        ParamId::TextInput,
        UiAlign::TopMid,
        0, 10,
        440, 40,
        c"Single-line command...",
        Some(c"SYS:STATUS"),
        31, // max length
    ),

    // 2. Multi-Line Text Area (e.g. system notes or log viewer)
    WidgetDescriptor::text_area(
        ParamId::TextArea,
        UiAlign::TopMid,
        0, 58,
        440, 75,
        c"Multi-line notes...",
        Some(c"ESP32-S3 LVGL v9\nTouch input active."),
        false, // multi-line
        127,   // max length
    ),

    // 3. On-Screen Virtual Keyboard (retargetable)
    WidgetDescriptor::keyboard(
        ParamId::Keyboard,
        ParamId::TextInput, // Initial target text field
        UiAlign::BottomMid,
        0, -5,
        460, 165,
    ),
];
```

#### Dynamic Keyboard Focus Switching

Rather than hardcoding a keyboard to a single input field forever:
1. When any `TextInput` or `TextArea` receives touch focus (`LV_EVENT_FOCUSED` or `LV_EVENT_CLICKED`), the C runtime invokes `on_textarea_focused`.
2. All active keyboards on the screen are dynamically re-routed to the focused target via `lv_keyboard_set_textarea(kb, ta)`.
3. Keystrokes, backspace, mode switching (`1#` numeric / symbol keys), and enter keys flow immediately into whichever field the user tapped.
4. When characters are edited, `LV_EVENT_VALUE_CHANGED` forwards zero-allocation Plain Old Data (POD) strings (`cmd.value.str_val: [u8; 32]`) over the lock-free bridge ring buffer to Rust's `SystemEngine`.

---

### Complete 34-Widget Suite (`EVERYTHING_SCHEMA`)

The **Everything** application integrates **all 34 widget types** across 7 dedicated functional tabs in a 480×480 layout:

| Tab | Name | Contained Widgets |
|---|---|---|
| **0** | **Controls** | `Switch`, `Led`, `Spinner`, `Arc` (dial), `Bar` (load), `Slider`, `Button`, `ImageButton`, `ArcLabel`, `Label` (cycles) |
| **1** | **Selectors** | `Checkbox`, `Dropdown`, `Roller`, `Spinbox`, `Scale` (analog needle gauge), `AnimImage` |
| **2** | **Inputs** | `TextInput` (single-line), `TextArea` (multi-line), `Keyboard` (virtual QWERTY), `MsgBox` (modal), `SpanGroup` |
| **3** | **Telemetry** | `Chart` (streaming waveform), `Line` (vector points), `Table` (diagnostics grid), `List` (interactive items) |
| **4** | **Tools** | `Calendar` (interactive date picker), `ButtonMatrix` (4×3 keypad), `Canvas` (100×100 RGB565 custom graphics), `Label` |
| **5** | **Media** | `Win` (movable window), `Barcode` (1D tag), `Image` (preview) |
| **6** | **System** | `Menu` (hierarchical settings), `QRCode` (Wi-Fi setup), `TileView` (horizontal swipeable 2D grid), `Label` (live transport stats) |

```rust
// Root Navigation Container: 7 Tabs across 480x480 screen
WidgetDescriptor::tabview(
    ParamId::NavTab,
    c"Controls\nSelectors\nInputs\nTelemetry\nTools\nMedia\nSystem",
    480, 480,
)
```

#### Automatic Inactivity Screen Timeout & Wake-on-Touch

To preserve power and display longevity on the Guition 480×480 screen:
- **Inactivity Timeout:** After 60 seconds (1 minute) with no touch events, `screen_saver_timer_cb` detects inactivity via `lv_display_get_inactive_time(disp) >= 60000` and disables the LED backlight (`bsp_backlight_set(0)`).
- **Wake-on-Touch:** When the screen is touched while dark, the Goodix GT911 touch digitizer detects the press, instantly turns the backlight back on to 100% (`bsp_backlight_set(100)`), and resets the inactivity timer.
- **Accidental Click Suppression:** The initial wake-up touch suppresses `LV_INDEV_STATE_PRESSED` forwarding, preventing unintended widget interactions in the dark.



