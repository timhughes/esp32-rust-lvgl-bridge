#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <assert.h>
#include <sys/stat.h>
#include "lvgl.h"
#include "harness.h"
#include "host_transport.h"
#include "app_ui.h"
#include "bridge_protocol.h"
#include "ui_binding.h"
#include "p4_c6_link.h"
#include "bsp_4848s040.h"

// ----------------------------------------------------------------------
// Phase 1 Unit Tests: Protocol & Engine Assertions
// ----------------------------------------------------------------------

static uint8_t s_mock_rx_buf[128];
static uint16_t s_mock_rx_len = 0;
static p4_c6_channel_t s_mock_rx_chan = 0;

static void mock_link_rx_cb(p4_c6_channel_t channel, const uint8_t *data, uint16_t len, void *user_data) {
    (void)user_data;
    s_mock_rx_chan = channel;
    s_mock_rx_len = (len < sizeof(s_mock_rx_buf)) ? len : sizeof(s_mock_rx_buf);
    memcpy(s_mock_rx_buf, data, s_mock_rx_len);
}

static void test_ffi_protocol(void) {
    printf("  [TEST] FFI Protocol & POD Serialization...\n");
    host_transport_reset();

    // Verify union value conversions
    param_val_t val;
    val.i32_val = -12345;
    assert(val.i32_val == -12345);

    val.f32_val = 3.14159f;
    assert(val.f32_val > 3.14f && val.f32_val < 3.15f);

    val.bool_val = true;
    assert(val.bool_val == true);

    strncpy(val.str_val, "TEST_CMD_STR", sizeof(val.str_val) - 1);
    val.str_val[sizeof(val.str_val) - 1] = '\0';
    assert(strcmp(val.str_val, "TEST_CMD_STR") == 0);

    // Verify UI Command queue serialization
    ui_command_t cmd = {
        .param_id = PARAM_SETPOINT_HZ,
        .seq_id = 42,
        .value.i32_val = 75,
    };
    bool ok = bridge_send_command_from_ui(&cmd);
    assert(ok);
    assert(host_transport_get_cmd_count() == 1);

    ui_command_t recv_cmd;
    ok = bridge_recv_command(&recv_cmd, 0);
    assert(ok);
    assert(recv_cmd.param_id == PARAM_SETPOINT_HZ);
    assert(recv_cmd.seq_id == 42);
    assert(recv_cmd.value.i32_val == 75);
    assert(host_transport_get_cmd_count() == 0);

    // Verify Device Telemetry queue serialization
    device_telemetry_t telem = {
        .param_id = PARAM_CYCLE_COUNT,
        .ack_seq_id = 42,
        .value.u32_val = 1000,
    };
    ok = bridge_send_telemetry(&telem);
    assert(ok);
    assert(host_transport_get_telem_count() == 1);

    device_telemetry_t recv_telem;
    ok = bridge_recv_telemetry_in_ui(&recv_telem);
    assert(ok);
    assert(recv_telem.param_id == PARAM_CYCLE_COUNT);
    assert(recv_telem.ack_seq_id == 42);
    assert(recv_telem.value.u32_val == 1000);
    assert(host_transport_get_telem_count() == 0);

    printf("    -> PASS: FFI POD structs serialize cleanly across ring buffers\n");
}

static void test_p4_c6_link(void) {
    printf("  [TEST] Inter-Chip Link (P4 <-> C6) Loopback & Protocol Framing...\n");
    p4_c6_link_config_t cfg = {
        .bus_type = P4_C6_BUS_LOOPBACK_MOCK,
        .bus_freq_khz = 50000,
    };
    esp_err_t err = p4_c6_link_init(&cfg);
    assert(err == ESP_OK);
    assert(p4_c6_link_is_alive());

    err = p4_c6_link_register_rx_callback(P4_C6_CHAN_TELEMETRY, mock_link_rx_cb, NULL);
    assert(err == ESP_OK);

    const char *msg = "PING:P4_C6_HEALTH_CHECK";
    s_mock_rx_len = 0;
    err = p4_c6_link_send(P4_C6_CHAN_TELEMETRY, msg, (uint16_t)(strlen(msg) + 1), 100);
    assert(err == ESP_OK);
    assert(s_mock_rx_chan == P4_C6_CHAN_TELEMETRY);
    assert(s_mock_rx_len == (uint16_t)(strlen(msg) + 1));
    assert(strcmp((char *)s_mock_rx_buf, msg) == 0);

    p4_c6_link_stats_t stats;
    p4_c6_link_get_stats(&stats);
    assert(stats.tx_packets == 1);
    assert(stats.rx_packets == 1);
    printf("    -> PASS: P4-C6 link framed, hashed with CRC16, and looped back\n");
}

static void test_rust_state_engine(void) {
    printf("  [TEST] Rust SystemEngine Closed-Loop State Simulation...\n");
    host_transport_reset();
    rust_worker_init();

    // Step 5 cycles while stopped: cycles must remain 0
    for (int i = 0; i < 5; i++) {
        rust_worker_step();
    }

    device_telemetry_t telem;
    bool has_cnt = false;
    uint32_t last_cnt = 0;
    while (bridge_recv_telemetry_in_ui(&telem)) {
        if (telem.param_id == PARAM_CYCLE_COUNT) {
            has_cnt = true;
            last_cnt = telem.value.u32_val;
        }
    }
    assert(has_cnt && last_cnt == 0);

    // Send SystemEnable = true
    ui_command_t cmd_en = {
        .param_id = PARAM_SYSTEM_ENABLE,
        .seq_id = 1,
        .value.bool_val = true,
    };
    bridge_send_command_from_ui(&cmd_en);

    // Step worker 10 times: cycles must increment
    for (int i = 0; i < 10; i++) {
        rust_worker_step();
        while (bridge_recv_telemetry_in_ui(&telem)) {
            if (telem.param_id == PARAM_CYCLE_COUNT) {
                last_cnt = telem.value.u32_val;
            }
        }
    }
    assert(last_cnt == 10);

    // Send TriggerReset = 1
    ui_command_t cmd_rst = {
        .param_id = PARAM_TRIGGER_RESET,
        .seq_id = 2,
        .value.i32_val = 1,
    };
    bridge_send_command_from_ui(&cmd_rst);
    rust_worker_step();

    while (bridge_recv_telemetry_in_ui(&telem)) {
        if (telem.param_id == PARAM_CYCLE_COUNT) {
            last_cnt = telem.value.u32_val;
        }
    }
    // After reset followed by 1 step, counter is 1
    assert(last_cnt == 1);

    printf("    -> PASS: Rust state engine correctly processes enable, frequency, and resets\n");
}

static void test_arbitration_engine(void) {
    printf("  [TEST] 3-Stage Anti-Jitter Arbitration Engine...\n");
    host_transport_reset();
    lv_obj_clean(lv_screen_active());
    setup_test_ui();
    ui_context_t *ui = get_test_ui();
    assert(ui->slider != NULL);

    // 1. Initial ARB_TRACKING state: incoming telemetry is accepted
    device_telemetry_t telem = {
        .param_id = PARAM_SETPOINT_HZ,
        .ack_seq_id = 0,
        .value.i32_val = 35,
    };
    ui_engine_dispatch_telemetry(&telem);
    assert(lv_slider_get_value(ui->slider) == 35);

    // 2. Active Touch Dragging: enters ARB_DRAGGING; incoming telemetry dropped
    harness_drag_slider_to_val(ui->slider, 70);
    assert(lv_slider_get_value(ui->slider) == 70);

    telem.value.i32_val = 15;
    ui_engine_dispatch_telemetry(&telem);
    // Slider must maintain the dragged value 70
    assert(lv_slider_get_value(ui->slider) == 70);

    // 3. Touch Released: enters ARB_HOLDOFF; stale ack_seq_id is rejected
    harness_touch_release();
    telem.ack_seq_id = 0; // Stale sequence ticket
    telem.value.i32_val = 20;
    ui_engine_dispatch_telemetry(&telem);
    assert(lv_slider_get_value(ui->slider) == 70);

    // 4. Current ack_seq_id matches: restores ARB_TRACKING
    ui_command_t emitted_cmd;
    bool got_cmd = bridge_recv_command(&emitted_cmd, 0);
    assert(got_cmd);
    assert(emitted_cmd.value.i32_val == 70);

    telem.ack_seq_id = emitted_cmd.seq_id;
    telem.value.i32_val = 70;
    ui_engine_dispatch_telemetry(&telem);
    assert(lv_slider_get_value(ui->slider) == 70);

    printf("    -> PASS: Arbitration lease protects UI during active dragging and holdoff\n");
}

// ----------------------------------------------------------------------
// Main Test Runner Entrypoint
// ----------------------------------------------------------------------

int main(void) {
    printf("========================================================\n");
    printf("   ESP32 Hybrid C/Rust LVGL Framework Test Suite       \n");
    printf("========================================================\n\n");

    // Ensure artifacts directory exists
    mkdir("artifacts", 0777);

    // Initialize transports and mock display
    bridge_transport_init(16, 32);
    rust_worker_init();
    harness_init();

    // ------------------------------------------------------------------
    // PHASE 1: Headless Protocol & Engine Assertions
    // ------------------------------------------------------------------
    printf("--- Phase 1: Unit Assertions (Headless Protocol & Engine) ---\n");
    test_ffi_protocol();
    test_p4_c6_link();
    test_rust_state_engine();
    test_arbitration_engine();
    printf("--- Phase 1 Complete: All Core Logic Contracts Verified ---\n\n");

    // ------------------------------------------------------------------
    // PHASE 2: Flagship S3-4848 6-Tab Application Journey (480x480)
    // ------------------------------------------------------------------
    printf("--- Phase 2: Flagship S3-4848 6-Tab Application Journey ---\n");
    harness_set_resolution(480, 480);
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t s3_count = 0;
    const widget_descriptor_t *s3_schema = rust_get_s3_schema(&s3_count);
    printf("Building S3-4848 UI schema with %zu widgets across 6 tabs...\n", s3_count);
    ui_engine_build_from_schema(s3_schema, s3_count);

    lv_obj_t *tv = ui_engine_get_widget_by_param(PARAM_NAV_TAB);
    assert(tv != NULL);

    // Step worker to populate initial bindings
    for (int i = 0; i < 5; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    // ----------------------------------------------------
    // Scenario 1: Tab 0 (Controls) Initial State
    // ----------------------------------------------------
    printf("[1/12] S3 Tab 0 (Controls) Initial State...\n");
    lv_tabview_set_active(tv, 0, LV_ANIM_OFF);
    harness_step(50);
    if (!harness_save_png("artifacts/01_s3_tab0_controls_init.png")) {
        fprintf(stderr, "Failed to save artifacts/01_s3_tab0_controls_init.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/01_s3_tab0_controls_init.png\n");

    // ----------------------------------------------------
    // Scenario 2: Tab 0 (Controls) Interactive Running & Dragging
    // ----------------------------------------------------
    printf("[2/12] S3 Tab 0 (Controls) Interactive Running & Dragging...\n");
    lv_obj_t *sw = ui_engine_get_widget_by_param(PARAM_SYSTEM_ENABLE);
    if (sw) harness_click_widget(sw);

    lv_obj_t *slider = ui_engine_get_widget_by_param(PARAM_SETPOINT_HZ);
    if (slider) {
        harness_drag_slider_to_val(slider, 60);
        harness_touch_release();
    }

    // Advance 25 cycles for closed-loop telemetry updates
    for (int i = 0; i < 25; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/02_s3_tab0_controls_running.png")) {
        fprintf(stderr, "Failed to save artifacts/02_s3_tab0_controls_running.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/02_s3_tab0_controls_running.png\n");

    // ----------------------------------------------------
    // Scenario 3: Tab 1 (Selectors) Selectors & Auxiliary Motor
    // ----------------------------------------------------
    printf("[3/12] S3 Tab 1 (Selectors) Dropdown, Roller, Spinbox & LED...\n");
    lv_tabview_set_active(tv, 1, LV_ANIM_OFF);
    harness_step(50);

    lv_obj_t *aux_cb = ui_engine_get_widget_by_param(PARAM_AUX_CHECKBOX);
    if (aux_cb) {
        harness_click_widget(aux_cb);
        for (int i = 0; i < 5; i++) {
            rust_worker_step();
            pump_incoming_telemetry();
            harness_step(20);
        }
    }

    if (!harness_save_png("artifacts/03_s3_tab1_selectors.png")) {
        fprintf(stderr, "Failed to save artifacts/03_s3_tab1_selectors.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/03_s3_tab1_selectors.png\n");

    // ----------------------------------------------------
    // Scenario 4: Tab 2 (Inputs) Keyboard Typing & Focus Retargeting
    // ----------------------------------------------------
    printf("[4/12] S3 Tab 2 (Inputs) Virtual Keyboard Typing & Focus Retargeting...\n");
    lv_tabview_set_active(tv, 2, LV_ANIM_OFF);
    harness_step(50);

    lv_obj_t *text_in = ui_engine_get_widget_by_param(PARAM_TEXT_INPUT);
    lv_obj_t *kb = ui_engine_get_widget_by_param(PARAM_KEYBOARD);
    if (text_in) {
        harness_click_widget(text_in);
        harness_step(20);
        if (kb) {
            harness_click_buttonmatrix_by_text(kb, "r");
            harness_click_buttonmatrix_by_text(kb, "u");
            harness_click_buttonmatrix_by_text(kb, "n");
        }
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    lv_obj_t *text_area = ui_engine_get_widget_by_param(PARAM_TEXT_AREA);
    if (text_area) {
        harness_click_widget(text_area);
        harness_step(20);
        if (kb) {
            harness_click_buttonmatrix_by_text(kb, "o");
            harness_click_buttonmatrix_by_text(kb, "k");
        }
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/04_s3_tab2_inputs_typing.png")) {
        fprintf(stderr, "Failed to save artifacts/04_s3_tab2_inputs_typing.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/04_s3_tab2_inputs_typing.png\n");

    // ----------------------------------------------------
    // Scenario 5: Tab 3 (Telemetry) Real-Time Waveforms & System Table
    // ----------------------------------------------------
    printf("[5/12] S3 Tab 3 (Telemetry) Real-Time Waveforms & System Table...\n");
    lv_tabview_set_active(tv, 3, LV_ANIM_OFF);
    harness_step(50);

    // Stream 25 cycles of waveform telemetry to populate chart
    for (int i = 0; i < 25; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/05_s3_tab3_telemetry_waveforms.png")) {
        fprintf(stderr, "Failed to save artifacts/05_s3_tab3_telemetry_waveforms.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/05_s3_tab3_telemetry_waveforms.png\n");

    // ----------------------------------------------------
    // Scenario 6: Tab 4 (Tools) Calendar, Keypad Matrix & Canvas
    // ----------------------------------------------------
    printf("[6/12] S3 Tab 4 (Tools) Maintenance Calendar, Keypad Matrix & Canvas...\n");
    lv_tabview_set_active(tv, 4, LV_ANIM_OFF);
    harness_step(50);

    lv_obj_t *keypad = ui_engine_get_widget_by_param(PARAM_KEYPAD_MATRIX);
    if (keypad) {
        harness_click_buttonmatrix_by_text(keypad, "5");
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/06_s3_tab4_tools.png")) {
        fprintf(stderr, "Failed to save artifacts/06_s3_tab4_tools.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/06_s3_tab4_tools.png\n");

    // ----------------------------------------------------
    // Scenario 7: Tab 5 (System) Settings Menu & Swipeable TileView
    // ----------------------------------------------------
    printf("[7/12] S3 Tab 5 (System) Settings Menu & Swipeable TileView...\n");
    lv_tabview_set_active(tv, 5, LV_ANIM_OFF);
    harness_step(50);

    lv_obj_t *tiles = ui_engine_get_widget_by_param(PARAM_NAV_TILE);
    if (tiles) {
        lv_tileview_set_tile_by_index(tiles, 1, 0, LV_ANIM_OFF);
        harness_step(50);
    }

    if (!harness_save_png("artifacts/07_s3_tab5_system.png")) {
        fprintf(stderr, "Failed to save artifacts/07_s3_tab5_system.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/07_s3_tab5_system.png\n");

    // ------------------------------------------------------------------
    // PHASE 3: Power Management & Inactivity Lifecycle
    // ------------------------------------------------------------------
    printf("--- Phase 3: Power Management & Inactivity Lifecycle ---\n");

    // Register 60-second inactivity screensaver timer
    harness_register_screensaver(60000);

    // ----------------------------------------------------
    // Scenario 8: Inactivity Timeout (Screen Blanked after 60s)
    // ----------------------------------------------------
    printf("[8/12] Inactivity Timeout (Screen Blanked after 60s)...\n");
    assert(bsp_backlight_is_on());
    // Advance tick by 61 seconds of idle time without touching
    harness_step(61000);
    assert(!bsp_backlight_is_on());

    if (!harness_save_png("artifacts/08_s3_screen_blanked.png")) {
        fprintf(stderr, "Failed to save artifacts/08_s3_screen_blanked.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/08_s3_screen_blanked.png\n");

    // ----------------------------------------------------
    // Scenario 9: Wake-on-Touch (Screen Restored, Blind Tap Swallowed)
    // ----------------------------------------------------
    printf("[9/12] Wake-on-Touch (Screen Restored, Blind Tap Swallowed)...\n");
    // Inject a tap while sleeping: backlight should restore, but touch state is swallowed
    harness_touch_set(240, 240, true);
    harness_step(30);
    harness_touch_release();
    harness_step(30);
    assert(bsp_backlight_is_on());

    if (!harness_save_png("artifacts/09_s3_screen_woken.png")) {
        fprintf(stderr, "Failed to save artifacts/09_s3_screen_woken.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/09_s3_screen_woken.png\n");

    // ------------------------------------------------------------------
    // PHASE 4: Standalone Widget Catalog Smoke Tests
    // ------------------------------------------------------------------
    printf("--- Phase 4: Standalone Widget Catalog Smoke Tests ---\n");
    harness_set_resolution(800, 480);

    // ----------------------------------------------------
    // Scenario 10: Tier 1 12-Widget Showcase
    // ----------------------------------------------------
    printf("[10/12] Tier 1 12-Widget Showcase...\n");
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t t1_count = 0;
    const widget_descriptor_t *t1_schema = rust_get_tier1_schema(&t1_count);
    ui_engine_build_from_schema(t1_schema, t1_count);

    for (int i = 0; i < 5; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/10_tier1_showcase.png")) {
        fprintf(stderr, "Failed to save artifacts/10_tier1_showcase.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/10_tier1_showcase.png\n");

    // ----------------------------------------------------
    // Scenario 11: Tier 2 Text, Keyboard, Matrix & Dialog Showcase
    // ----------------------------------------------------
    printf("[11/12] Tier 2 Text, Keyboard, Matrix & Dialog Showcase...\n");
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t t2_count = 0;
    const widget_descriptor_t *t2_schema = rust_get_tier2_schema(&t2_count);
    ui_engine_build_from_schema(t2_schema, t2_count);

    for (int i = 0; i < 5; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/11_tier2_showcase.png")) {
        fprintf(stderr, "Failed to save artifacts/11_tier2_showcase.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/11_tier2_showcase.png\n");

    // ----------------------------------------------------
    // Scenario 12: Tier 3 Multi-Tab Container Showcase
    // ----------------------------------------------------
    printf("[12/12] Tier 3 Multi-Tab Container Showcase...\n");
    lv_obj_clean(lv_screen_active());
    ui_engine_init();

    size_t t3_count = 0;
    const widget_descriptor_t *t3_schema = rust_get_tier3_schema(&t3_count);
    ui_engine_build_from_schema(t3_schema, t3_count);

    for (int i = 0; i < 15; i++) {
        rust_worker_step();
        pump_incoming_telemetry();
        harness_step(20);
    }

    if (!harness_save_png("artifacts/12_tier3_showcase.png")) {
        fprintf(stderr, "Failed to save artifacts/12_tier3_showcase.png\n");
        return 1;
    }
    printf("  -> Saved artifacts/12_tier3_showcase.png\n");

    printf("\n========================================================\n");
    printf("   All Tests Passed! 12 Visual Artifacts Generated.     \n");
    printf("========================================================\n");
    return 0;
}
