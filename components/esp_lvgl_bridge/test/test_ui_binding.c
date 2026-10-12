#include "unity.h"
#include "bridge_protocol.h"
#include "ui_binding.h"
#include "lvgl.h"

// Set up minimal LVGL environment for component tests
static void init_test_lvgl_display(void) {
    static bool lvgl_initialized = false;
    if (!lvgl_initialized) {
        lv_init();

        static uint8_t buf1[480 * 20 * 2];
        lv_display_t *disp = lv_display_create(480, 480);
        lv_display_set_buffers(disp, buf1, NULL, sizeof(buf1), LV_DISPLAY_RENDER_MODE_PARTIAL);
        lvgl_initialized = true;
    }
}

TEST_CASE("ui_binding null telemetry and empty schema guards", "[esp_lvgl_bridge]") {
    init_test_lvgl_display();
    ui_engine_init();

    // 1. Dispatching NULL telemetry must be safe no-op
    ui_engine_dispatch_telemetry(NULL);

    // 2. Building from NULL or 0-count schema must be safe no-op
    ui_engine_build_from_schema(NULL, 0);
    widget_descriptor_t dummy = {0};
    ui_engine_build_from_schema(&dummy, 0);

    // 3. Pumping empty bridge telemetry should not crash
    ui_engine_pump_telemetry();
}

TEST_CASE("ui_binding param lookup bounds and nonexistent IDs", "[esp_lvgl_bridge]") {
    init_test_lvgl_display();
    ui_engine_init();

    // PARAM_NONE must return NULL
    TEST_ASSERT_NULL(ui_engine_get_widget_by_param(PARAM_NONE));

    // Arbitrary unmapped param IDs must return NULL
    TEST_ASSERT_NULL(ui_engine_get_widget_by_param(9999));
    TEST_ASSERT_NULL(ui_engine_get_widget_by_param(12345));
}

TEST_CASE("ui_binding schema construction and widget lookup", "[esp_lvgl_bridge]") {
    init_test_lvgl_display();
    ui_engine_init();

    param_id_t p_sw = 101;
    param_id_t p_slider = 102;
    param_id_t p_btn = 103;
    param_id_t p_lbl = 104;

    const widget_descriptor_t schema[] = {
        {
            .widget_type = WIDGET_TYPE_SWITCH,
            .param_id = p_sw,
            .align = UI_ALIGN_CENTER,
            .initial_val = 0,
            .label_primary = "ON",
            .label_secondary = "OFF",
        },
        {
            .widget_type = WIDGET_TYPE_SLIDER,
            .param_id = p_slider,
            .align = UI_ALIGN_CENTER,
            .min_val = 0,
            .max_val = 100,
            .initial_val = 50,
            .label_primary = "%ld%%",
        },
        {
            .widget_type = WIDGET_TYPE_BUTTON,
            .param_id = p_btn,
            .align = UI_ALIGN_CENTER,
            .initial_val = 1,
            .label_primary = "Click Me",
        },
        {
            .widget_type = WIDGET_TYPE_LABEL,
            .param_id = p_lbl,
            .align = UI_ALIGN_CENTER,
            .label_primary = "Status: %s",
        },
    };

    ui_engine_build_from_schema(schema, sizeof(schema) / sizeof(schema[0]));

    // Check all registered widgets can be retrieved
    lv_obj_t *obj_sw = ui_engine_get_widget_by_param(p_sw);
    TEST_ASSERT_NOT_NULL(obj_sw);

    lv_obj_t *obj_slider = ui_engine_get_widget_by_param(p_slider);
    TEST_ASSERT_NOT_NULL(obj_slider);

    lv_obj_t *obj_btn = ui_engine_get_widget_by_param(p_btn);
    TEST_ASSERT_NOT_NULL(obj_btn);

    lv_obj_t *obj_lbl = ui_engine_get_widget_by_param(p_lbl);
    TEST_ASSERT_NOT_NULL(obj_lbl);

    // Unmapped param still returns NULL
    TEST_ASSERT_NULL(ui_engine_get_widget_by_param(999));
}

TEST_CASE("ui_binding telemetry dispatch updates widget state", "[esp_lvgl_bridge]") {
    init_test_lvgl_display();
    ui_engine_init();

    param_id_t p_slider = 201;
    param_id_t p_sw = 202;
    param_id_t p_lbl = 203;

    const widget_descriptor_t schema[] = {
        {
            .widget_type = WIDGET_TYPE_SLIDER,
            .param_id = p_slider,
            .align = UI_ALIGN_CENTER,
            .min_val = 0,
            .max_val = 100,
            .initial_val = 10,
            .label_primary = "Val: %ld",
        },
        {
            .widget_type = WIDGET_TYPE_SWITCH,
            .param_id = p_sw,
            .align = UI_ALIGN_CENTER,
            .initial_val = 0,
            .label_primary = "ON",
            .label_secondary = "OFF",
        },
        {
            .widget_type = WIDGET_TYPE_LABEL,
            .param_id = p_lbl,
            .align = UI_ALIGN_CENTER,
            .label_primary = "Mode: %s",
        },
    };

    ui_engine_build_from_schema(schema, sizeof(schema) / sizeof(schema[0]));

    lv_obj_t *slider = ui_engine_get_widget_by_param(p_slider);
    lv_obj_t *sw = ui_engine_get_widget_by_param(p_sw);
    lv_obj_t *lbl = ui_engine_get_widget_by_param(p_lbl);

    TEST_ASSERT_NOT_NULL(slider);
    TEST_ASSERT_NOT_NULL(sw);
    TEST_ASSERT_NOT_NULL(lbl);

    // Initial slider value
    TEST_ASSERT_EQUAL_INT32(10, lv_slider_get_value(slider));

    // Dispatch telemetry for slider (value = 85)
    device_telemetry_t telem_slider = {
        .param_id = p_slider,
        .ack_seq_id = 1,
        .value.i32_val = 85,
    };
    ui_engine_dispatch_telemetry(&telem_slider);
    TEST_ASSERT_EQUAL_INT32(85, lv_slider_get_value(slider));

    // Initial switch state (OFF)
    TEST_ASSERT_FALSE(lv_obj_has_state(sw, LV_STATE_CHECKED));

    // Dispatch telemetry for switch (value = true)
    device_telemetry_t telem_sw = {
        .param_id = p_sw,
        .ack_seq_id = 2,
        .value.bool_val = true,
    };
    ui_engine_dispatch_telemetry(&telem_sw);
    TEST_ASSERT_TRUE(lv_obj_has_state(sw, LV_STATE_CHECKED));

    // Dispatch string telemetry for label
    device_telemetry_t telem_lbl = {
        .param_id = p_lbl,
        .ack_seq_id = 3,
        .value.str_val = "ARMED",
    };
    ui_engine_dispatch_telemetry(&telem_lbl);
    TEST_ASSERT_EQUAL_STRING("Mode: ARMED", lv_label_get_text(lbl));
}

TEST_CASE("ui_binding dynamic widget flag and state changes", "[esp_lvgl_bridge]") {
    init_test_lvgl_display();
    ui_engine_init();

    param_id_t p_btn = 301;
    const widget_descriptor_t schema[] = {
        {
            .widget_type = WIDGET_TYPE_BUTTON,
            .param_id = p_btn,
            .align = UI_ALIGN_CENTER,
            .initial_val = 1,
            .label_primary = "Test Button",
        },
    };

    ui_engine_build_from_schema(schema, sizeof(schema) / sizeof(schema[0]));
    lv_obj_t *btn = ui_engine_get_widget_by_param(p_btn);
    TEST_ASSERT_NOT_NULL(btn);

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
    // By default button is not hidden, not disabled
    TEST_ASSERT_FALSE(lv_obj_has_flag(btn, LV_OBJ_FLAG_HIDDEN));
    TEST_ASSERT_FALSE(lv_obj_has_state(btn, LV_STATE_DISABLED));

    // Dispatch PARAM_WIDGET_FLAGS with WIDGET_FLAG_HIDDEN | WIDGET_FLAG_DISABLED
    device_telemetry_t telem_flags = {
        .param_id = PARAM_WIDGET_FLAGS,
        .ack_seq_id = (uint32_t)p_btn,
        .value.u32_val = WIDGET_FLAG_HIDDEN | WIDGET_FLAG_DISABLED,
    };
    ui_engine_dispatch_telemetry(&telem_flags);

    TEST_ASSERT_TRUE(lv_obj_has_flag(btn, LV_OBJ_FLAG_HIDDEN));
    TEST_ASSERT_TRUE(lv_obj_has_state(btn, LV_STATE_DISABLED));

    // Dispatch clear flags: WIDGET_FLAG_VISIBLE | WIDGET_FLAG_ENABLED
    telem_flags.value.u32_val = WIDGET_FLAG_VISIBLE | WIDGET_FLAG_ENABLED;
    ui_engine_dispatch_telemetry(&telem_flags);

    TEST_ASSERT_FALSE(lv_obj_has_flag(btn, LV_OBJ_FLAG_HIDDEN));
    TEST_ASSERT_FALSE(lv_obj_has_state(btn, LV_STATE_DISABLED));
#pragma GCC diagnostic pop
}

TEST_CASE("ui_binding re-initialization and clean re-build", "[esp_lvgl_bridge]") {
    init_test_lvgl_display();

    // First cycle
    ui_engine_init();
    const widget_descriptor_t schema1[] = {
        {
            .widget_type = WIDGET_TYPE_BUTTON,
            .param_id = 401,
            .align = UI_ALIGN_CENTER,
            .label_primary = "Cycle 1",
        },
    };
    ui_engine_build_from_schema(schema1, 1);
    TEST_ASSERT_NOT_NULL(ui_engine_get_widget_by_param(401));

    // Re-init cleans lookups
    ui_engine_init();
    TEST_ASSERT_NULL(ui_engine_get_widget_by_param(401));

    // Second cycle
    const widget_descriptor_t schema2[] = {
        {
            .widget_type = WIDGET_TYPE_BUTTON,
            .param_id = 402,
            .align = UI_ALIGN_CENTER,
            .label_primary = "Cycle 2",
        },
    };
    ui_engine_build_from_schema(schema2, 1);
    TEST_ASSERT_NULL(ui_engine_get_widget_by_param(401));
    TEST_ASSERT_NOT_NULL(ui_engine_get_widget_by_param(402));
}
