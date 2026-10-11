#ifndef HARNESS_H
#define HARNESS_H

#include <stdint.h>
#include <stdbool.h>
#include "lvgl.h"

#ifdef __cplusplus
extern "C" {
#endif

#define HARNESS_HOR_RES 800
#define HARNESS_VER_RES 480

/**
 * @brief Initialize the headless offscreen LVGL display and mock touch driver.
 */
void harness_init(void);

/**
 * @brief Advance LVGL tick and run timer handler by elapsed_ms.
 */
void harness_step(uint32_t elapsed_ms);

/**
 * @brief Run multiple simulation steps over a duration.
 */
void harness_step_loop(uint32_t duration_ms, uint32_t step_interval_ms);

/**
 * @brief Inject mock touch state at specific screen coordinates.
 */
void harness_touch_set(int32_t x, int32_t y, bool pressed);

/**
 * @brief Lift touch (release) at current coordinates without moving pointer.
 */
void harness_touch_release(void);

/**
 * @brief Simulate a touch press and immediate release (click) on a widget.
 */
void harness_click_widget(lv_obj_t *obj);

/**
 * @brief Simulate a drag gesture on a slider to a target value.
 */
void harness_drag_slider_to_val(lv_obj_t *slider, int32_t target_val);

/**
 * @brief Save the current offscreen framebuffer to a PNG file.
 */
bool harness_save_png(const char *filename);

/**
 * @brief Set display resolution (e.g. 480x480 for S3-4848 or 800x480 for wide screens).
 */
void harness_set_resolution(int32_t hor_res, int32_t ver_res);

/**
 * @brief Get current horizontal resolution.
 */
int32_t harness_get_hor_res(void);

/**
 * @brief Get current vertical resolution.
 */
int32_t harness_get_ver_res(void);

/**
 * @brief Simulate a touch click on a specific button within an LVGL button matrix or keyboard by label text.
 */
bool harness_click_buttonmatrix_by_text(lv_obj_t *bm, const char *text);

/**
 * @brief Simulate a touch click on a specific button within an LVGL button matrix by button index.
 */
bool harness_click_buttonmatrix_by_id(lv_obj_t *bm, uint32_t btn_id);

/**
 * @brief Set mock display backlight brightness (0 to 100).
 */
void bsp_backlight_set(uint8_t percent);

/**
 * @brief Check if mock display backlight is currently turned on.
 */
bool bsp_backlight_is_on(void);

/**
 * @brief Register inactivity screensaver timer on the harness display.
 */
void harness_register_screensaver(uint32_t timeout_ms);

/**
 * @brief Get pointer to raw RGB888 framebuffer.
 */
const uint8_t *harness_get_framebuffer(void);

#ifdef __cplusplus
}
#endif

#endif // HARNESS_H
