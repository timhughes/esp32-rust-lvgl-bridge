#include "unity.h"
#include "bsp_4848s040.h"

TEST_CASE("bsp resolution invariants and ops table integrity", "[bsp_4848s040]") {
    TEST_ASSERT_EQUAL_INT(480, BSP_LCD_H_RES);
    TEST_ASSERT_EQUAL_INT(480, BSP_LCD_V_RES);

    const esp_lvgl_bsp_ops_t *ops = bsp_4848s040_get_ops();
    TEST_ASSERT_NOT_NULL(ops);
    TEST_ASSERT_NOT_NULL(ops->board_name);
    TEST_ASSERT_NOT_NULL(ops->init_display);
    TEST_ASSERT_NOT_NULL(ops->init_touch);
    TEST_ASSERT_NOT_NULL(ops->set_backlight);
    TEST_ASSERT_NOT_NULL(ops->backlight_is_on);
}

TEST_CASE("bsp backlight state tracking and duty cycle", "[bsp_4848s040]") {
    const esp_lvgl_bsp_ops_t *ops = bsp_4848s040_get_ops();

    // Turn backlight OFF
    ops->set_backlight(0);
    TEST_ASSERT_FALSE(ops->backlight_is_on());

    // Turn backlight ON
    ops->set_backlight(100);
    TEST_ASSERT_TRUE(ops->backlight_is_on());

    // 50% brightness is still ON
    ops->set_backlight(50);
    TEST_ASSERT_TRUE(ops->backlight_is_on());

    // Turn backlight OFF again
    ops->set_backlight(0);
    TEST_ASSERT_FALSE(ops->backlight_is_on());
}
