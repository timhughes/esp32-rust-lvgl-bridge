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

## Generated Artifacts (`test_harness/artifacts/`)
* **`01_initial_state.png`**: System stopped, slider at 10 Hz, cycle counter at 0.
* **`02_switch_toggled.png`**: Switch toggled to RUNNING, background worker begins stepping.
* **`03_slider_dragging.png`**: Touch injected dragging slider to 60 Hz.
* **`04_slider_released.png`**: Touch lifted; 500ms holdoff lease active.
* **`05_slider_telemetry_settled.png`**: Holdoff expired; label confirms 60 Hz (Actual).
* **`06_counter_reset.png`**: Push button clicked; cycle count atomically clears to 0 and resumes.
* **`07_tier1_showcase.png`**: 480×480 screen with all Tier 1 widgets active (Arc, Switch, LED, Bar, Spinner, Counter, Reset Button).
