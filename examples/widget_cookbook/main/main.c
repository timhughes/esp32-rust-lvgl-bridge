#include "esp_lvgl_bridge.h"

#ifdef ESP_PLATFORM
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_timer.h"
#include "esp_log.h"
#if CONFIG_IDF_TARGET_ESP32S3
#include "bsp_4848s040.h"
#endif
#endif

static const char *TAG = "COOKBOOK_APP";

static uint32_t bsp_lvgl_tick(void) {
    return (uint32_t)(esp_timer_get_time() / 1000);
}

#if CONFIG_IDF_TARGET_ESP32S3
static const esp_lvgl_bsp_ops_t *s_bsp = NULL;
#endif

static void lvgl_ui_task(void *pvParams) {
    (void)pvParams;

    ESP_LOGI(TAG, "LVGL UI task started on Core %d", xPortGetCoreID());
    lv_init();
    lv_tick_set_cb(bsp_lvgl_tick);

#if CONFIG_IDF_TARGET_ESP32S3
    // Acquire the Guition ESP32-S3 BSP operations interface
    s_bsp = bsp_4848s040_get_ops();
    ESP_LOGI(TAG, "Initializing display via BSP operations (%s)...", s_bsp->board_name);
    lv_display_t *disp = s_bsp->init_display();
    s_bsp->init_touch(disp);

    // Build UI from Rust Tier 1 showcase schema
    size_t count = 0;
    const widget_descriptor_t *schema = rust_get_tier1_schema(&count);
    ESP_LOGI(TAG, "Constructing UI from Rust Tier 1 schema (%d widgets)...", (int)count);
    ui_engine_init();
    ui_engine_build_from_schema(schema, count);
    ESP_LOGI(TAG, "UI construction complete!");
#endif

    while (1) {
        ui_engine_pump_telemetry();
        uint32_t delay_ms = lv_timer_handler();
        if (delay_ms < 5) delay_ms = 5;
        if (delay_ms > 20) delay_ms = 20;
        vTaskDelay(pdMS_TO_TICKS(delay_ms));
    }
}

static void rust_worker_task(void *pvParams) {
    (void)pvParams;

    ESP_LOGI(TAG, "Rust worker task started on Core %d", xPortGetCoreID());
    rust_worker_init();

    while (1) {
        rust_worker_step();
        vTaskDelay(pdMS_TO_TICKS(20)); // 50 Hz control loop
    }
}

void app_main(void) {
    ESP_LOGI(TAG, "Starting Widget Cookbook Example Application...");

    // Initialize lock-free ring buffers (16-slot commands, 32-slot telemetry)
    bridge_transport_init(16, 32);

    // Pin Rust domain logic to Core 0
    xTaskCreatePinnedToCore(
        rust_worker_task,
        "rust_worker",
        4096,
        NULL,
        5,
        NULL,
        0
    );

    // Pin LVGL UI rendering task to Core 1
    xTaskCreatePinnedToCore(
        lvgl_ui_task,
        "lvgl_ui",
        8192,
        NULL,
        5,
        NULL,
        1
    );
}
