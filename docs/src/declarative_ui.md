# Declarative UI Engine

The framework replaces procedural C UI assembly with a **data-driven declarative schema** defined in pure Rust.

## Core Schema Structs

The schema consists of static array descriptors exposed over the C FFI boundary:

```rust
#[repr(C)]
pub struct WidgetDescriptor {
    pub widget_type: WidgetType,
    pub param_id: u32,
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    pub min_val: i32,
    pub max_val: i32,
    pub default_val: i32,
    pub label_text: *const c_char,
}
```

## Supported Widgets (Tier 1)
1. **Labels (`WIDGET_LABEL`)**: Text display with dynamic `sprintf` formatting based on bound numeric parameters.
2. **Sliders (`WIDGET_SLIDER`)**: Interactive touch drag controls with holdoff leases.
3. **Switches (`WIDGET_SWITCH`)**: Boolean toggles emitting instant state transitions.
4. **Push Buttons (`WIDGET_BUTTON`)**: Momentary touch triggers (e.g., reset, start, stop).
5. **Arcs (`WIDGET_ARC`)**: Circular dial knobs and rotational setpoints.
6. **Progress Bars (`WIDGET_BAR`)**: Linear state gauges (battery, buffer health, progress).
7. **LED Indicators (`WIDGET_LED`)**: Circular on/off status LEDs with brightness/color control.
8. **Spinners (`WIDGET_SPINNER`)**: Indeterminate activity spinners.

## Example Rust Schema Definition
```rust
pub static S3_SCHEMA: &[WidgetDescriptor] = &[
    WidgetDescriptor {
        widget_type: WidgetType::Label,
        param_id: 0,
        x: 0, y: 15, w: 480, h: 40,
        min_val: 0, max_val: 0, default_val: 0,
        label_text: b"ESP32-S3 Smart Display\0".as_ptr() as *const c_char,
    },
    WidgetDescriptor {
        widget_type: WidgetType::Arc,
        param_id: PARAM_SETPOINT_HZ,
        x: 140, y: 70, w: 200, h: 200,
        min_val: 1, max_val: 120, default_val: 10,
        label_text: core::ptr::null(),
    },
    // ...
];
```
The C runtime initializes the entire UI hierarchy dynamically from this schema table with a single call to `ui_engine_build_from_schema(schema, count)`.
