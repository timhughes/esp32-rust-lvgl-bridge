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
The test suite executes 11 discrete scenarios sequentially against the in-memory display engine:
1. **Scenarios 1–6 (Baseline Interactive Pipeline)**: Exercises the 3-stage arbitration engine (`TRACKING` $\rightarrow$ `DRAGGING` $\rightarrow$ `HOLDOFF`), verifying sequence ID tracking, temporal leases, and atomic counter reset.
2. **Scenario 7 (Tier 1 Showcase)**: Instantiates and binds all 12 scalar/discrete widgets on a single 480×480 screen.
3. **Scenario 8 (Tier 2 Showcase)**: Tests text input, keyboard focus, button matrices, analog dial scales, and modal alert dialogs.
4. **Scenarios 9–11 (Tier 3 Multi-Tab Container & Streams)**: Validates hierarchical container routing across a 3-tab `TabView`, cycling active tabs via synthetic touch and updating real-time streaming charts, diagnostic tables, date pickers, item lists, and canvas graphics.

## Generated Artifacts (`test_harness/artifacts/`)
* **`01_initial_state.png`**: System stopped, slider at default 10 Hz, cycle counter at 0.
* **`02_switch_toggled.png`**: Switch toggled to RUNNING, background worker begins stepping.
* **`03_slider_dragging.png`**: Touch injected dragging slider to 60 Hz (`60 Hz (Setting)`).
* **`04_slider_released.png`**: Touch lifted; 500ms holdoff lease active suppressing intermediate telemetry.
* **`05_slider_telemetry_settled.png`**: Holdoff expired; label confirms `60 Hz (Actual)`, cycles incrementing.
* **`06_counter_reset.png`**: Push button clicked; cycle count atomically clears to 0 and resumes.
* **`07_tier1_showcase.png`**: 480×480 screen with all Tier 1 widgets active (Arc, Switch, LED, Bar, Spinner, Checkbox, Dropdown, Roller, Spinbox).
* **`08_tier2_showcase.png`**: 480×480 screen with Tier 2 widgets active (TextArea pre-filled, On-Screen Keyboard, Numeric/Keypad ButtonMatrix, Analog Speedometer Scale, MsgBox alert overlay).
* **`09_tier3_telemetry.png`**: Tier 3 TabView with Tab 0 ("Telemetry") active: real-time streaming waveform Chart, interactive Arc knob, and system status Bar.
* **`10_tier3_diagnostics.png`**: Tier 3 TabView with Tab 1 ("Diagnostics") active: multi-row system status Table (`Module`, `Status`, `Core 0`, `Core 1`, `PSRAM`) and scrollable Item List.
* **`11_tier3_config.png`**: Tier 3 TabView with Tab 2 ("Config") active: interactive Calendar date picker and raw-pixel graphics Canvas.
