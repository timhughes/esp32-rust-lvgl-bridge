/**
 * @file esp_lvgl_bridge.h
 * @brief Umbrella header for the ESP32 Rust/C LVGL Framework.
 *
 * Provides bidirectional communication between FreeRTOS C tasks (LVGL UI)
 * and no_std Rust state machines, declarative UI schema binding for 26 LVGL
 * widgets, 3-stage user-interaction arbitration, and co-processor link framing.
 */

#ifndef ESP_LVGL_BRIDGE_H
#define ESP_LVGL_BRIDGE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "bridge_protocol.h"
#include "ui_binding.h"
#include "p4_c6_link.h"

#ifdef __cplusplus
}
#endif

#endif // ESP_LVGL_BRIDGE_H
