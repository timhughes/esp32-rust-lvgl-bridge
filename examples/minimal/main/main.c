/**
 * @file main.c
 * @brief Minimal Example Application using the ESP32 Rust/LVGL Framework.
 */

#include "esp_lvgl_bridge.h"
#include "bsp_4848s040.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_timer.h"
#include "esp_log.h"

static const char *TAG = "MINIMAL_APP";

// FFI declarations exported by minimal-app Rust crate
extern void rust_minimal_init(void);
extern void rust_minimal_step(void);
extern const widget_descriptor_t *rust_get_minimal_schema(size_t *out_count);

static uint32_t lvgl_tick_cb(void) {
    return (uint32_t)(esp_timer_get_time() / 1000);
}

static void lvgl_ui_task(void *pvParams) {
    (void)pvParams;
    ESP_LOGI(TAG, "Starting LVGL UI task on Core %d...", xPortGetCoreID());

    lv_init();
    lv_tick_set_cb(lvgl_tick_cb);

    // Initialize display and touch via BSP ops
    const esp_lvgl_bsp_ops_t *bsp = bsp_4848s040_get_ops();
    ESP_LOGI(TAG, "Initializing display via %s...", bsp->board_name);
    lv_display_t *disp = bsp->init_display();
    bsp->init_touch(disp);

    // Initialize UI engine and build scene graph from Rust schema
    ui_engine_init();
    size_t count = 0;
    const widget_descriptor_t *schema = rust_get_minimal_schema(&count);
    ESP_LOGI(TAG, "Building scene graph from Rust schema (%d widgets)...", (int)count);
    ui_engine_build_from_schema(schema, count);

    while (1) {
        ui_engine_pump_telemetry();
        uint32_t delay_ms = lv_timer_handler();
        vTaskDelay(pdMS_TO_TICKS(delay_ms > 10 ? 10 : (delay_ms < 1 ? 1 : delay_ms)));
    }
}

static void rust_worker_task(void *pvParams) {
    (void)pvParams;
    ESP_LOGI(TAG, "Starting Rust worker task on Core %d...", xPortGetCoreID());

    rust_minimal_init();
    while (1) {
        rust_minimal_step();
        vTaskDelay(pdMS_TO_TICKS(20)); // 50 Hz deterministic loop
    }
}

void app_main(void) {
    ESP_LOGI(TAG, "=== ESP32 Rust + LVGL Minimal Starter Example ===");

    // Initialize FreeRTOS command and telemetry queues
    bridge_transport_init(32, 64);

    // Core 0: Rust closed-loop worker (8KB stack)
    xTaskCreatePinnedToCore(rust_worker_task, "rust_worker", 8192, NULL, 5, NULL, 0);

    // Core 1: LVGL GUI & touch loop (8KB stack)
    xTaskCreatePinnedToCore(lvgl_ui_task, "lvgl_task", 8192, NULL, 5, NULL, 1);
}
