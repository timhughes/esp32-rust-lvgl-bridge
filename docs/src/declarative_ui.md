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
18. **`TabView`**: Multi-screen tabbed interface with top navigation headers (*Telemetry*, *Diagnostics*, *Config*).
19. **`TileView`**: 2D swipeable grid container.
20. **`Menu`**: Hierarchical multi-level settings drill-down menu.
21. **`List`**: Scrollable action list with items.
22. **`Chart`**: Real-time time-series oscilloscope / waveform chart updated via streaming telemetry.
23. **`Table`**: Multi-row, multi-column tabular grid for system status, Wi-Fi scans, and diagnostic logs.
24. **`Calendar`**: Month calendar viewer and date picker.
25. **`Canvas`**: Raw pixel graphics buffer for custom drawing.

---

## Example Rust Schema Definitions

### Multi-Tab Container (`TIER3_SHOWCASE_SCHEMA`)

```rust
pub static TIER3_SHOWCASE_SCHEMA: &[WidgetDescriptor] = &[
    // 1. Root Navigation Container
    WidgetDescriptor::tabview(
        ParamId::NavTab,
        c"Telemetry\nDiagnostics\nConfig",
        760,
        460,
    ),
    // 2. Tab 0 (Telemetry): Real-Time Waveform Chart
    WidgetDescriptor::chart(
        ParamId::TelemetryChart,
        ParamId::NavTab,
        0, // Mounted in Tab 0
        UiAlign::Center,
        0, -70,
        640, 220,
        0, 100,
    ),
    // 3. Tab 0 (Telemetry): Channel Status List
    WidgetDescriptor::list(
        ParamId::ItemList,
        ParamId::NavTab,
        0, // Mounted in Tab 0
        UiAlign::BottomMid,
        0, -15,
        640, 110,
        c"Channel A (Active)\nChannel B (Standby)\nBus Voltage (3.3V)",
    ),
    // 4. Tab 1 (Diagnostics): Status Table
    WidgetDescriptor::table(
        ParamId::DataTable,
        ParamId::NavTab,
        1, // Mounted in Tab 1
        UiAlign::Center,
        0, 0,
        500, 240,
        (4 << 16) | 2, // 4 rows, 2 columns
        c"Module;Status;Core 0;OK;Core 1;OK;PSRAM;8MB",
    ),
    // 5. Tab 2 (Config): Interactive Calendar
    WidgetDescriptor::calendar(
        ParamId::CalendarDate,
        ParamId::NavTab,
        2, // Mounted in Tab 2
        UiAlign::Center,
        0, 0,
        320, 280,
    ),
];
```
