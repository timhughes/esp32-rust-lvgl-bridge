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
static uint32_t bsp_lvgl_tick(void) {
    return (uint32_t)(esp_timer_get_time() / 1000);
}

#if CONFIG_IDF_TARGET_ESP32S3
#define SCREEN_TIMEOUT_MS (60 * 1000)

static void screen_saver_timer_cb(lv_timer_t *timer) {
    lv_display_t *disp = (lv_display_t *)lv_timer_get_user_data(timer);
    uint32_t inactive_ms = lv_display_get_inactive_time(disp);
    if (bsp_backlight_is_on() && inactive_ms >= SCREEN_TIMEOUT_MS) {
        ESP_LOGI("APP", "Inactivity timeout (%lu ms >= 60s). Turning display backlight OFF.", (unsigned long)inactive_ms);
        bsp_backlight_set(0);
    }
}
#endif

static void lvgl_ui_task(void *pvParams) {
    (void)pvParams;

    ESP_LOGI("APP", "lvgl_ui_task started on Core %d", xPortGetCoreID());
    lv_init();
    lv_tick_set_cb(bsp_lvgl_tick);

#if CONFIG_IDF_TARGET_ESP32S3
    ESP_LOGI("APP", "Initializing S3-4848S040 display & touch...");
    lv_display_t *disp = bsp_display_init();
    bsp_touch_init(disp);

    size_t count = 0;
    const widget_descriptor_t *schema = rust_get_s3_schema(&count);
    ESP_LOGI("APP", "Building UI from Rust S3 schema (%d widgets)...", (int)count);
    ui_engine_init();
    ui_engine_build_from_schema(schema, count);
    ESP_LOGI("APP", "UI schema construction complete!");

    // Register 1-minute inactivity screensaver timer (checks once every second)
    lv_timer_create(screen_saver_timer_cb, 1000, disp);
    ESP_LOGI("APP", "Registered 60-second screen timeout timer.");
#else
    setup_test_ui();
#endif

    while (1) {
        ui_engine_pump_telemetry();
        uint32_t delay_ms = lv_timer_handler();
        vTaskDelay(pdMS_TO_TICKS(delay_ms > 10 ? 10 : (delay_ms < 1 ? 1 : delay_ms)));
    }
}

static void rust_worker_task(void *pvParams) {
    (void)pvParams;
    ESP_LOGI("APP", "rust_worker_task started on Core %d", xPortGetCoreID());
    rust_worker_init();
    ESP_LOGI("APP", "rust_worker_init() succeeded!");
    uint32_t step_count = 0;
    while (1) {
        rust_worker_step();
        step_count++;
        if (step_count % 100 == 0) {
            ESP_LOGI("APP", "Rust control loop alive: %lu steps (Core %d)", (unsigned long)step_count, xPortGetCoreID());
        }
        vTaskDelay(pdMS_TO_TICKS(20));
    }
}


void app_main(void) {
    ESP_LOGI("APP", "Starting app_main() on Core %d...", xPortGetCoreID());
    bridge_transport_init(16, 32);

    // If running on ESP32-P4, initialize co-processor link to C6
#if CONFIG_IDF_TARGET_ESP32P4
    p4_c6_link_config_t link_cfg = {
        .bus_type = P4_C6_BUS_SDIO_3_0,
        .bus_freq_khz = 50000,
    };
    p4_c6_link_init(&link_cfg);
#endif

    // Core 0: Rust closed-loop control and state engine (8KB stack)
    xTaskCreatePinnedToCore(rust_worker_task, "rust_worker", 8192, NULL, 5, NULL, 0);
    // Core 1: LVGL GUI rendering and touch handling (8KB stack)
    xTaskCreatePinnedToCore(lvgl_ui_task, "lvgl_task", 8192, NULL, 5, NULL, 1);
    ESP_LOGI("APP", "Worker tasks spawned successfully!");
}

