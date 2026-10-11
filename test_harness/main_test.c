#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <sys/stat.h>
#include "lvgl.h"
#include "harness.h"
#include "host_transport.h"
#include "../main/include/app_ui.h"
#include "../main/include/bridge_protocol.h"
#include "../main/include/ui_binding.h"

int main(void) {
    printf("=== Starting Headless Host Visual Test Suite ===\n");

    // Ensure artifacts directory exists
    mkdir("artifacts", 0777);

    // 1. Initialize transports and Rust worker
    bridge_transport_init(16, 32);
    rust_worker_init();

    // 2. Initialize offscreen LVGL display and test UI
    harness_init();
    setup_test_ui();
    ui_context_t *ui = get_test_ui();

    // ----------------------------------------------------
    // Scenario 1: Initial state
    // System stopped, slider at default 10 Hz, cycle count 0
    // ----------------------------------------------------
    printf("[1/7] Capturing Scenario 1: Initial state...\n");
    harness_step(50);
    if (!harness_save_png("artifacts/01_initial_state.png")) {
        fprintf(stderr, "Failed to save artifacts/01_initial_state.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/01_initial_state.png\n");

    // ----------------------------------------------------
    // Scenario 2: Switch toggled
    // Toggle switch to RUNNING, worker begins stepping
    // ----------------------------------------------------
    printf("[2/7] Capturing Scenario 2: Switch toggled to RUNNING...\n");
    harness_click_widget(ui->power_switch);

    // Step worker and pump telemetry for a few cycles
    for (int i = 0; i < 10; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }
    if (!harness_save_png("artifacts/02_switch_toggled.png")) {
        fprintf(stderr, "Failed to save artifacts/02_switch_toggled.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/02_switch_toggled.png\n");

    // ----------------------------------------------------
    // Scenario 3: Slider dragging
    // Touch injected dragging slider to 60 Hz; label reads (Setting)
    // ----------------------------------------------------
    printf("[3/7] Capturing Scenario 3: Slider dragging to 60 Hz...\n");
    harness_drag_slider_to_val(ui->slider, 60);

    // Step worker in background (telemetry dropped by active touch lease)
    rust_worker_step();
    pump_incoming_telemetry();
    harness_step(20);

    if (!harness_save_png("artifacts/03_slider_dragging.png")) {
        fprintf(stderr, "Failed to save artifacts/03_slider_dragging.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/03_slider_dragging.png\n");

    // ----------------------------------------------------
    // Scenario 4: Slider released
    // Touch lifted; holdoff lease actively rejecting old telemetry
    // ----------------------------------------------------
    printf("[4/7] Capturing Scenario 4: Slider released (Holdoff active)...\n");
    harness_touch_release();

    // Step worker a few times; intermediate telemetry is rejected by holdoff
    for (int i = 0; i < 3; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/04_slider_released.png")) {
        fprintf(stderr, "Failed to save artifacts/04_slider_released.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/04_slider_released.png\n");

    // ----------------------------------------------------
    // Scenario 5: Slider telemetry settled
    // Holdoff expired; label confirms 60 Hz (Actual), cycles running
    // ----------------------------------------------------
    printf("[5/7] Capturing Scenario 5: Slider telemetry settled at 60 Hz...\n");
    // Advance past 500ms holdoff timer and let worker reach 60 Hz
    for (int i = 0; i < 60; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/05_slider_telemetry_settled.png")) {
        fprintf(stderr, "Failed to save artifacts/05_slider_telemetry_settled.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/05_slider_telemetry_settled.png\n");

    // ----------------------------------------------------
    // Scenario 6: Counter reset
    // Push button clicked; cycle count atomically clears to 0
    // ----------------------------------------------------
    printf("[6/7] Capturing Scenario 6: Counter reset button clicked...\n");
    harness_click_widget(ui->reset_btn);

    // Worker receives reset command and resets cycle counter
    rust_worker_step();
    pump_incoming_telemetry();
    harness_step(20);

    if (!harness_save_png("artifacts/06_counter_reset.png")) {
        fprintf(stderr, "Failed to save artifacts/06_counter_reset.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/06_counter_reset.png\n");

    // ----------------------------------------------------
    // Scenario 7: Tier 1 Showcase
    // Declarative schema with all Tier 1 widgets: Arc, Bar, Checkbox, LED,
    // Spinner, Dropdown, Roller, Spinbox
    // ----------------------------------------------------
    printf("[7/7] Capturing Scenario 7: Tier 1 Widget Showcase...\n");
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t t1_count = 0;
    const widget_descriptor_t *t1_schema = rust_get_tier1_schema(&t1_count);
    printf("  -> Building Tier 1 UI schema with %zu widgets...\n", t1_count);
    ui_engine_build_from_schema(t1_schema, t1_count);

    // Step worker and pump telemetry to populate all initial values
    for (int i = 0; i < 5; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    // Toggle Aux Checkbox to verify two-way data binding with LED
    lv_obj_t *aux_cb = ui_engine_get_widget_by_param(PARAM_AUX_CHECKBOX);
    if (aux_cb) {
        harness_click_widget(aux_cb);
        for (int i = 0; i < 5; i++) {
            rust_worker_step();
            pump_incoming_telemetry();
            harness_step(20);
        }
    }

    if (!harness_save_png("artifacts/07_tier1_showcase.png")) {
        fprintf(stderr, "Failed to save artifacts/07_tier1_showcase.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/07_tier1_showcase.png\n");

    // ----------------------------------------------------
    // Scenario 8: Tier 2 Showcase
    // Declarative schema with all Tier 2 widgets: TextArea,
    // Keyboard, ButtonMatrix, Scale, MsgBox
    // ----------------------------------------------------
    printf("[8/9] Capturing Scenario 8: Tier 2 Widget Showcase...\n");
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t t2_count = 0;
    const widget_descriptor_t *t2_schema = rust_get_tier2_schema(&t2_count);
    printf("  -> Building Tier 2 UI schema with %zu widgets...\n", t2_count);
    ui_engine_build_from_schema(t2_schema, t2_count);

    // Step worker and pump telemetry for 10 cycles to calibrate scale and buffers
    for (int i = 0; i < 10; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    // Interact with Keypad Matrix
    lv_obj_t *keypad = ui_engine_get_widget_by_param(PARAM_KEYPAD_MATRIX);
    if (keypad) {
        harness_click_widget(keypad);
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/08_tier2_showcase.png")) {
        fprintf(stderr, "Failed to save artifacts/08_tier2_showcase.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/08_tier2_showcase.png\n");

    // ----------------------------------------------------
    // Scenario 9: Tier 3 Showcase (Tab 0: Telemetry - Chart & List)
    // ----------------------------------------------------
    printf("[9/11] Capturing Scenario 9: Tier 3 Tab 0 (Telemetry)...\n");
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t t3_count = 0;
    const widget_descriptor_t *t3_schema = rust_get_tier3_schema(&t3_count);
    printf("  -> Building Tier 3 UI schema with %zu widgets...\n", t3_count);
    ui_engine_build_from_schema(t3_schema, t3_count);

    // Step worker across 20 cycles to stream real-time chart waveform data
    for (int i = 0; i < 20; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(30);
    }

    if (!harness_save_png("artifacts/09_tier3_telemetry.png")) {
        fprintf(stderr, "Failed to save artifacts/09_tier3_telemetry.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/09_tier3_telemetry.png\n");

    // ----------------------------------------------------
    // Scenario 10: Tier 3 Tab 1 (Diagnostics - Table)
    // ----------------------------------------------------
    printf("[10/11] Capturing Scenario 10: Tier 3 Tab 1 (Diagnostics - Table)...\n");
    lv_obj_t *tv = ui_engine_get_widget_by_param(PARAM_NAV_TAB);
    if (tv) {
        lv_tabview_set_active(tv, 1, LV_ANIM_OFF);
        harness_step(50);
    }
    if (!harness_save_png("artifacts/10_tier3_diagnostics.png")) {
        fprintf(stderr, "Failed to save artifacts/10_tier3_diagnostics.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/10_tier3_diagnostics.png\n");

    // ----------------------------------------------------
    // Scenario 11: Tier 3 Tab 2 (Config - Calendar)
    // ----------------------------------------------------
    printf("[11/11] Capturing Scenario 11: Tier 3 Tab 2 (Config - Calendar)...\n");
    if (tv) {
        lv_tabview_set_active(tv, 2, LV_ANIM_OFF);
        harness_step(50);
    }
    if (!harness_save_png("artifacts/11_tier3_config.png")) {
        fprintf(stderr, "Failed to save artifacts/11_tier3_config.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/11_tier3_config.png\n");

    printf("=== Test Suite Complete: All 11 Artifacts Successfully Generated ===\n");
    return 0;
}
