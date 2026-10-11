#ifndef BSP_4848S040_H
#define BSP_4848S040_H

#ifdef ESP_PLATFORM
#include "esp_err.h"
#else
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_FAIL -1
#endif
#include "lvgl.h"

#ifdef __cplusplus
extern "C" {
#endif

#define BSP_LCD_H_RES       480
#define BSP_LCD_V_RES       480

/**
 * @brief Initialize ST7701S RGB panel and register with LVGL v9.
 * @return lv_display_t* pointer to registered LVGL display or NULL on failure.
 */
lv_display_t *bsp_display_init(void);

/**
 * @brief Initialize Goodix GT911 touch panel and register with LVGL v9.
 * @param disp Pointer to registered LVGL display.
 * @return lv_indev_t* pointer to registered LVGL input device or NULL on failure.
 */
lv_indev_t *bsp_touch_init(lv_display_t *disp);

/**
 * @brief Set display backlight brightness.
 * @param percent Brightness percentage (0 to 100).
 */
void bsp_backlight_set(uint8_t percent);

/**
 * @brief Check if display backlight is currently turned on.
 * @return true if backlight is on, false if off.
 */
bool bsp_backlight_is_on(void);

#include "esp_lvgl_bsp.h"

/**
 * @brief Get the hardware operations table for the Guition ESP32-S3-4848S040 board.
 * @return const esp_lvgl_bsp_ops_t* pointer to static operations table.
 */
const esp_lvgl_bsp_ops_t *bsp_4848s040_get_ops(void);

#ifdef __cplusplus
}
#endif

#endif // BSP_4848S040_H
