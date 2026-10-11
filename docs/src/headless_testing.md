# Headless Visual QA Suite

The test suite runs entirely offscreen on host developer machines (Linux/macOS) without requiring hardware attached.

## How It Works
* **RGB888 In-Memory Canvas**: LVGL renders directly into an allocated buffer in host RAM.
* **Deterministic Virtual Time**: The test runner steps `lv_timer_handler()` with explicit virtual clock increments.
* **Synthetic Touch Injection**: Mouse/touch coordinates and press/release events are injected programmatically into the LVGL input device driver.
* **PNG Output**: After each UI scenario, frames are dumped directly to standard PNG images using `stb_image_write.h`.

## Running the Tests
```bash
cd test_harness
make clean && make run
```

## Test Execution Workflow
The test suite executes 4 structured testing phases:
1. **Phase 1: Headless Protocol & Engine Unit Assertions**:
   - `test_ffi_protocol`: Verifies POD struct sizing, union value conversions, string buffer safety, and ring buffer queue operations.
   - `test_p4_c6_link`: Verifies P4 <-> C6 loopback mock, packet framing, CRC16 checksum calculation, and sequence tracking.
   - `test_rust_state_engine`: Verifies closed-loop simulation (halted when stopped, incrementing when enabled, frequency scaling, and atomic resets).
   - `test_arbitration_engine`: Verifies 3-stage arbitration lease (`ARB_TRACKING` $\rightarrow$ `ARB_DRAGGING` $\rightarrow$ `ARB_HOLDOFF`), rejecting telemetry during drags and stale tickets during holdoffs.
2. **Phase 2: Flagship S3-4848 6-Tab Application Journey (480×480)**:
   - Exercises the complete 27-widget production application across all 6 tabs (Controls, Selectors, Inputs, Telemetry, Tools, System).
   - Injects virtual keyboard keystrokes, verifies dynamic focus retargeting between single-line and multi-line text fields.
   - Streams live sine waveform oscilloscope data into the chart and updates diagnostic tables.
3. **Phase 3: Power Management & Inactivity Lifecycle**:
   - Verifies 60-second inactivity blanking (`bsp_backlight_set(0)`).
   - Verifies wake-on-touch restoration and blind first-tap suppression (ignoring the wake tap to prevent accidental clicks).
4. **Phase 4: Standalone Widget Catalog Smoke Tests (800×480)**:
   - Regression smoke tests covering isolated Tier 1, Tier 2, and Tier 3 showcase layouts.

## Generated Visual Artifacts (`test_harness/artifacts/`)

| Scenario | Snapshot | Description |
|---|---|---|
| **01** | `01_s3_tab0_controls_init.png` | S3-4848 Tab 0 (Controls): Initial stopped state, 10 Hz slider, 0 cycles |
| **02** | `02_s3_tab0_controls_running.png` | S3-4848 Tab 0 (Controls): RUNNING state, slider dragged to 60 Hz, active cycles |
| **03** | `03_s3_tab1_selectors.png` | S3-4848 Tab 1 (Selectors): Dropdown, Roller, Spinbox & illuminated Aux LED |
| **04** | `04_s3_tab2_inputs_typing.png` | S3-4848 Tab 2 (Inputs): Virtual Keyboard focus retargeting & character entry |
| **05** | `05_s3_tab3_telemetry_waveforms.png` | S3-4848 Tab 3 (Telemetry): Real-time waveform oscilloscope & table registers |
| **06** | `06_s3_tab4_tools.png` | S3-4848 Tab 4 (Tools): Maintenance Calendar, 4×3 PIN Keypad & Canvas |
| **07** | `07_s3_tab5_system.png` | S3-4848 Tab 5 (System): Hierarchical Settings Menu & swipeable TileView |
| **08** | `08_s3_screen_blanked.png` | Power Management: 60-second inactivity timeout blanking display backlight |
| **09** | `09_s3_screen_woken.png` | Power Management: Wake-on-touch restoration with blind first-tap suppression |
| **10** | `10_tier1_showcase.png` | Isolated Showcase: All 12 Tier 1 scalar widgets active simultaneously |
| **11** | `11_tier2_showcase.png` | Isolated Showcase: Tier 2 inputs, virtual keyboard, button matrix & scale |
| **12** | `12_tier3_showcase.png` | Isolated Showcase: Tier 3 multi-tab container with charts, tables & lists |
