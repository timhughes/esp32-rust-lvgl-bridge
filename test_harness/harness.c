#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"
#include "harness.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint8_t s_framebuffer[HARNESS_HOR_RES * HARNESS_VER_RES * 3];
static lv_display_t *s_disp = NULL;
static lv_indev_t *s_indev = NULL;

static int32_t s_touch_x = 0;
static int32_t s_touch_y = 0;
static bool s_touch_pressed = false;

static void harness_flush_cb(lv_display_t *disp, const lv_area_t *area, uint8_t *px_map) {
    (void)area;
    (void)px_map;
    lv_display_flush_ready(disp);
}

static void harness_touch_read_cb(lv_indev_t *indev, lv_indev_data_t *data) {
    (void)indev;
    data->point.x = s_touch_x;
    data->point.y = s_touch_y;
    data->state = s_touch_pressed ? LV_INDEV_STATE_PRESSED : LV_INDEV_STATE_RELEASED;
}

void harness_init(void) {
    lv_init();

    // Create offscreen display with RGB888 buffer
    s_disp = lv_display_create(HARNESS_HOR_RES, HARNESS_VER_RES);
    lv_display_set_color_format(s_disp, LV_COLOR_FORMAT_RGB888);
    lv_display_set_buffers(s_disp, s_framebuffer, NULL, sizeof(s_framebuffer), LV_DISPLAY_RENDER_MODE_DIRECT);
    lv_display_set_flush_cb(s_disp, harness_flush_cb);

    // Create mock pointer indev
    s_indev = lv_indev_create();
    lv_indev_set_type(s_indev, LV_INDEV_TYPE_POINTER);
    lv_indev_set_display(s_indev, s_disp);
    lv_indev_set_read_cb(s_indev, harness_touch_read_cb);

    // Initial render pass
    harness_step(50);
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

    int res = stbi_write_png(filename, HARNESS_HOR_RES, HARNESS_VER_RES, 3, s_framebuffer, HARNESS_HOR_RES * 3);
    return res != 0;
}

const uint8_t *harness_get_framebuffer(void) {
    return s_framebuffer;
}
