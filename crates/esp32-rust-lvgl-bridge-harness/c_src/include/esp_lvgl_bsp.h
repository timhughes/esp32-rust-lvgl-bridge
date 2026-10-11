/**
 * @file esp_lvgl_bsp.h
 * @brief Generic Hardware Board Support Package (BSP) Interface for LVGL displays.
 */

#ifndef ESP_LVGL_BSP_H
#define ESP_LVGL_BSP_H

#include <stdint.h>
#include <stdbool.h>
#include "lvgl.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Hardware Board Support Package operations interface.
 *
 * Board-specific display drivers (ST7701S, ILI9341, MIPI-DSI, etc.) and
 * touch digitizers (GT911, FT6236, CST816, etc.) implement this struct
 * to decouple board hardware initialization from the LVGL scene graph and bridge.
 */
typedef struct {
    /** @brief Human-readable name of the board or display module (e.g. "Guition ESP32-S3-4848S040"). */
    const char *board_name;

    /** @brief Initialize panel bus, timing, and register the LVGL display. Returns lv_display_t*. */
    lv_display_t *(*init_display)(void);

    /** @brief Initialize touch controller (I2C/SPI) and register the LVGL indev. Returns lv_indev_t*. */
    lv_indev_t   *(*init_touch)(lv_display_t *disp);

    /** @brief Set display backlight brightness (0 = off/blank, 100 = full brightness). */
    void          (*set_backlight)(uint8_t percent);

    /** @brief Check whether the backlight is currently turned on. */
    bool          (*backlight_is_on)(void);
} esp_lvgl_bsp_ops_t;

#ifdef __cplusplus
}
#endif

#endif // ESP_LVGL_BSP_H
