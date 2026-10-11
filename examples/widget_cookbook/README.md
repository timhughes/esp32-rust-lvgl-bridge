# Widget Cookbook: Declarative UI Recipe Catalog

The **Widget Cookbook** is a comprehensive reference and recipe catalog demonstrating how to define, layout, and bind all 27 supported LVGL widgets using `bridge-core`'s declarative schema system.

Developers building their own applications can browse these recipes and copy/paste widget declarations directly into their projects.

---

## Catalog Recipes

### Recipe 1: Tier 1 Scalar Controls & Indicators ([`TIER1_SHOWCASE_SCHEMA`](crates/widget-cookbook/src/lib.rs#L30))
Contains 12 foundational widgets suitable for control panels and instrument clusters:
1. **Toggle Switch**: Bistable Run/Stop control with custom state labels.
2. **Checkbox**: Binary option toggle.
3. **Status LED**: Visual on/off indicator.
4. **Spinner**: Infinite activity indicator.
5. **Arc Knob**: Circular rotational dial with min/max, step size, and unit formatting.
6. **Slider**: Linear horizontal range slider with anti-jitter arbitration.
7. **Progress Bar**: Read-only bar indicating operational percentage.
8. **Dropdown**: Modal list selector.
9. **Roller**: Scrollable cylindrical option wheel.
10. **Button**: Momentary action trigger.
11. **Label**: Live telemetry text readout.
12. **Spinbox**: Multi-digit numeric step controller.

### Recipe 2: Tier 2 Text, Keypads, Scales & Modals ([`TIER2_SHOWCASE_SCHEMA`](crates/widget-cookbook/src/lib.rs#L148))
Contains interactive text entry and modal feedback components:
1. **Single-Line Text Input**: Edit single strings with placeholder text.
2. **Multi-Line Text Area**: Scrollable multi-line log or note viewer.
3. **Button Matrix Keypad**: 3×4 or custom grid matrix with click event routing.
4. **Analog Scale Dial**: Radial meter with tick divisions and needle position.
5. **On-Screen Keyboard**: Virtual keyboard attached to focused text inputs.
6. **Message Box**: Non-blocking pop-up modal dialog for system notices and warnings.

### Recipe 3: Tier 3 Multi-Tab Containers & Visualizations ([`TIER3_SHOWCASE_SCHEMA`](crates/widget-cookbook/src/lib.rs#L210))
Contains complex container layouts and advanced visualizations:
1. **TabView Container**: Multi-page container with tab navigation bar.
2. **Waveform Chart**: Real-time streaming line chart with circular FIFO buffering.
3. **Structured Table**: Multi-row, multi-column diagnostic grid.
4. **Interactive Calendar**: Date picker with month/year navigation.
5. **Scrollable List**: Icon/text item list.

---

## Building and Flashing the Cookbook

### 1. Build the Rust Static Library
```bash
cargo build --release --target xtensa-esp32s3-none-elf -p widget-cookbook
```

### 2. Build and Flash ESP-IDF Firmware
```bash
idf.py -DIDF_TARGET=esp32s3 -C examples/widget_cookbook build
idf.py -p /dev/ttyUSB0 -C examples/widget_cookbook flash monitor
```
