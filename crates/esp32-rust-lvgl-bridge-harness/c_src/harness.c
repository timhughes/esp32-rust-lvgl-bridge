#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"
#include "harness.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "widgets/buttonmatrix/lv_buttonmatrix_private.h"

#define HARNESS_MAX_HOR_RES 800
#define HARNESS_MAX_VER_RES 480

static uint8_t s_framebuffer[HARNESS_MAX_HOR_RES * HARNESS_MAX_VER_RES * 3];
static int32_t s_hor_res = HARNESS_HOR_RES;
static int32_t s_ver_res = HARNESS_VER_RES;
static lv_display_t *s_disp = NULL;
static lv_indev_t *s_indev = NULL;
static lv_timer_t *s_screensaver_timer = NULL;
static uint32_t s_screensaver_timeout_ms = 60000;

static int32_t s_touch_x = 0;
static int32_t s_touch_y = 0;
static bool s_touch_pressed = false;
static bool s_backlight_on = true;

void bsp_backlight_set(uint8_t percent) {
    s_backlight_on = (percent > 0);
}

bool bsp_backlight_is_on(void) {
    return s_backlight_on;
}

static void screen_saver_timer_cb(lv_timer_t *timer) {
    lv_display_t *disp = (lv_display_t *)lv_timer_get_user_data(timer);
    uint32_t inactive_ms = lv_display_get_inactive_time(disp);
    if (s_backlight_on && inactive_ms >= s_screensaver_timeout_ms) {
        bsp_backlight_set(0);
    }
}

void harness_register_screensaver(uint32_t timeout_ms) {
    s_screensaver_timeout_ms = timeout_ms;
    if (!s_screensaver_timer && s_disp) {
        s_screensaver_timer = lv_timer_create(screen_saver_timer_cb, 500, s_disp);
    }
}

static void harness_flush_cb(lv_display_t *disp, const lv_area_t *area, uint8_t *px_map) {
    (void)area;
    (void)px_map;
    lv_display_flush_ready(disp);
}

static void harness_touch_read_cb(lv_indev_t *indev, lv_indev_data_t *data) {
    (void)indev;
    if (s_touch_pressed) {
        if (!s_backlight_on) {
            // Wake on touch: restore backlight, reset inactivity timer, swallow blind tap!
            s_backlight_on = true;
            lv_display_trigger_activity(s_disp);
            data->state = LV_INDEV_STATE_RELEASED;
            return;
        }
        data->point.x = s_touch_x;
        data->point.y = s_touch_y;
        data->state = LV_INDEV_STATE_PRESSED;
    } else {
        data->state = LV_INDEV_STATE_RELEASED;
    }
}

void harness_init(void) {
    lv_init();

    // Create offscreen display with RGB888 buffer
    s_disp = lv_display_create(s_hor_res, s_ver_res);
    lv_display_set_color_format(s_disp, LV_COLOR_FORMAT_RGB888);
    lv_display_set_buffers(s_disp, s_framebuffer, NULL, sizeof(s_framebuffer), LV_DISPLAY_RENDER_MODE_DIRECT);
    lv_display_set_flush_cb(s_disp, harness_flush_cb);

    // Create mock pointer indev
    s_indev = lv_indev_create();
    lv_indev_set_type(s_indev, LV_INDEV_TYPE_POINTER);
    lv_indev_set_display(s_indev, s_disp);
    lv_indev_set_read_cb(s_indev, harness_touch_read_cb);

    s_backlight_on = true;

    // Initial render pass
    harness_step(50);
}

void harness_set_resolution(int32_t hor_res, int32_t ver_res) {
    if (hor_res <= 0 || ver_res <= 0) return;
    if (hor_res > HARNESS_MAX_HOR_RES || ver_res > HARNESS_MAX_VER_RES) return;

    s_hor_res = hor_res;
    s_ver_res = ver_res;
    if (s_disp) {
        lv_display_set_resolution(s_disp, hor_res, ver_res);
        lv_obj_set_size(lv_screen_active(), hor_res, ver_res);
        lv_obj_invalidate(lv_screen_active());
        harness_step(50);
    }
}

int32_t harness_get_hor_res(void) {
    return s_hor_res;
}

int32_t harness_get_ver_res(void) {
    return s_ver_res;
}

void harness_step(uint32_t elapsed_ms) {
    lv_tick_inc(elapsed_ms);
    if (s_indev) {
        lv_indev_read(s_indev);
    }
    lv_timer_handler();
}

void harness_step_loop(uint32_t duration_ms, uint32_t step_interval_ms) {
    uint32_t elapsed = 0;
    while (elapsed < duration_ms) {
        harness_step(step_interval_ms);
        elapsed += step_interval_ms;
    }
}

void harness_touch_set(int32_t x, int32_t y, bool pressed) {
    s_touch_x = x;
    s_touch_y = y;
    s_touch_pressed = pressed;
}

void harness_touch_release(void) {
    s_touch_pressed = false;
    harness_step(30);
}

void harness_click_widget(lv_obj_t *obj) {
    if (!obj) return;
    lv_obj_update_layout(lv_screen_active());
    lv_area_t coords;
    lv_obj_get_coords(obj, &coords);
    int32_t cx = (coords.x1 + coords.x2) / 2;
    int32_t cy = (coords.y1 + coords.y2) / 2;

    // Press
    harness_touch_set(cx, cy, true);
    harness_step(30);

    // Release
    harness_touch_release();
    harness_step(30);
}

bool harness_click_buttonmatrix_by_id(lv_obj_t *bm, uint32_t btn_id) {
    if (!bm) return false;
    lv_obj_update_layout(lv_screen_active());
    struct _lv_buttonmatrix_t *bm_p = (struct _lv_buttonmatrix_t *)bm;
    if (btn_id >= bm_p->btn_cnt || !bm_p->button_areas) return false;

    lv_area_t obj_coords;
    lv_obj_get_coords(bm, &obj_coords);
    lv_area_t btn_area = bm_p->button_areas[btn_id];

    int32_t cx = obj_coords.x1 + (btn_area.x1 + btn_area.x2) / 2;
    int32_t cy = obj_coords.y1 + (btn_area.y1 + btn_area.y2) / 2;

    harness_touch_set(cx, cy, true);
    harness_step(30);

    harness_touch_release();
    harness_step(30);
    return true;
}

bool harness_click_buttonmatrix_by_text(lv_obj_t *bm, const char *text) {
    if (!bm || !text) return false;
    lv_obj_update_layout(lv_screen_active());
    struct _lv_buttonmatrix_t *bm_p = (struct _lv_buttonmatrix_t *)bm;
    if (!bm_p->button_areas) return false;

    for (uint32_t i = 0; i < bm_p->btn_cnt; i++) {
        const char *t = lv_buttonmatrix_get_button_text(bm, i);
        if (t && strcmp(t, text) == 0) {
            return harness_click_buttonmatrix_by_id(bm, i);
        }
    }
    return false;
}

void harness_drag_slider_to_val(lv_obj_t *slider, int32_t target_val) {
    if (!slider) return;
    lv_obj_update_layout(lv_screen_active());
    lv_area_t coords;
    lv_obj_get_coords(slider, &coords);

    int32_t min = lv_slider_get_min_value(slider);
    int32_t max = lv_slider_get_max_value(slider);
    int32_t cur = lv_slider_get_value(slider);
    int32_t width = coords.x2 - coords.x1;
    int32_t cy = (coords.y1 + coords.y2) / 2;

    int32_t start_x = coords.x1 + ((cur - min) * width) / (max - min);
    int32_t end_x = coords.x1 + ((target_val - min) * width) / (max - min);

    // Press at current position
    harness_touch_set(start_x, cy, true);
    harness_step(20);

    // Intermediate drag steps
    int32_t steps = 5;
    for (int i = 1; i <= steps; i++) {
        int32_t x = start_x + ((end_x - start_x) * i) / steps;
        harness_touch_set(x, cy, true);
        harness_step(20);
    }
}

bool harness_save_png(const char *filename) {
    if (!filename) return false;

    // Force full redraw before snapshotting
    lv_obj_invalidate(lv_screen_active());
    lv_timer_handler();

    if (!s_backlight_on) {
        // Display backlight is turned off: clear framebuffer to black for visual QA
        memset(s_framebuffer, 0, s_hor_res * s_ver_res * 3);
    }

    int res = stbi_write_png(filename, s_hor_res, s_ver_res, 3, s_framebuffer, s_hor_res * 3);
    return res != 0;
}

const uint8_t *harness_get_framebuffer(void) {
    return s_framebuffer;
}

#include "app_ui.h"
#include "ui_binding.h"
#include "bridge_protocol.h"

static ui_context_t s_ui;

ui_context_t *get_test_ui(void) {
    return &s_ui;
}

void pump_incoming_telemetry(void) {
    ui_engine_pump_telemetry();
}

void setup_test_ui(void) {
    ui_engine_init();

    size_t count = 0;
    const widget_descriptor_t *schema = rust_get_ui_schema(&count);
    ui_engine_build_from_schema(schema, count);

    s_ui.power_switch = ui_engine_get_widget_by_param(PARAM_SYSTEM_ENABLE);
    s_ui.slider       = ui_engine_get_widget_by_param(PARAM_SETPOINT_HZ);
    s_ui.reset_btn    = ui_engine_get_widget_by_param(PARAM_TRIGGER_RESET);
    s_ui.counter_label= ui_engine_get_widget_by_param(PARAM_CYCLE_COUNT);
}

