#include <stdio.h>
#include "unity.h"
#include "unity_test_runner.h"
#include "esp_log.h"

void app_main(void) {
    printf("==========================================\n");
    printf("Running ESP32 LVGL Framework Component Tests\n");
    printf("==========================================\n");

    UNITY_BEGIN();
    unity_run_all_tests();
    UNITY_END();
}
