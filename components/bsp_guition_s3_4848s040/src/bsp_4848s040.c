#include "bsp_4848s040.h"
#include <stdio.h>
#include <string.h>

static bool s_backlight_on = true;

bool bsp_backlight_is_on(void) {
    return s_backlight_on;
}

#ifdef ESP_PLATFORM
#include "esp_log.h"
#include "esp_err.h"
#include "esp_heap_caps.h"
#include "driver/gpio.h"
#include "driver/i2c.h"
#include "esp_lcd_panel_io.h"
#include "esp_lcd_panel_ops.h"
#include "esp_lcd_panel_rgb.h"
#include "esp_lcd_st7701.h"
#include "esp_lcd_panel_io_additions.h"
#include "esp_lcd_touch_gt911.h"

static const char *TAG = "bsp_4848s040";

// --- Hardware Pin Definitions ---
#define LCD_IO_BL              38

// 3-wire SPI for ST7701S command initialization
#define LCD_IO_SPI_CS          39
#define LCD_IO_SPI_SCLK        48
#define LCD_IO_SPI_SDA         47

// RGB Panel
#define LCD_IO_RGB_DE          18
#define LCD_IO_RGB_PCLK        21
#define LCD_IO_RGB_VSYNC       17
#define LCD_IO_RGB_HSYNC       16

// RGB Data (RGB565)
// Red
#define LCD_IO_RGB_DATA0       11 // R0
#define LCD_IO_RGB_DATA1       12 // R1
#define LCD_IO_RGB_DATA2       13 // R2
#define LCD_IO_RGB_DATA3       14 // R3
#define LCD_IO_RGB_DATA4       0  // R4
// Green
#define LCD_IO_RGB_DATA5       8  // G0
#define LCD_IO_RGB_DATA6       20 // G1
#define LCD_IO_RGB_DATA7       3  // G2
#define LCD_IO_RGB_DATA8       46 // G3
#define LCD_IO_RGB_DATA9       9  // G4
#define LCD_IO_RGB_DATA10      10 // G5
// Blue
#define LCD_IO_RGB_DATA11      4  // B0
#define LCD_IO_RGB_DATA12      5  // B1
#define LCD_IO_RGB_DATA13      6  // B2
#define LCD_IO_RGB_DATA14      7  // B3
#define LCD_IO_RGB_DATA15      15 // B4

// Touch I2C
#define TOUCH_I2C_NUM          I2C_NUM_0
#define TOUCH_I2C_SDA          19
#define TOUCH_I2C_SCL          45

static esp_lcd_panel_handle_t s_panel_handle = NULL;
static esp_lcd_touch_handle_t s_touch_handle = NULL;

static void lvgl_flush_cb(lv_display_t *disp, const lv_area_t *area, uint8_t *px_map) {
    esp_lcd_panel_handle_t panel = (esp_lcd_panel_handle_t)lv_display_get_user_data(disp);
    int offsetx1 = area->x1;
    int offsetx2 = area->x2;
    int offsety1 = area->y1;
    int offsety2 = area->y2;
    esp_lcd_panel_draw_bitmap(panel, offsetx1, offsety1, offsetx2 + 1, offsety2 + 1, px_map);
    lv_display_flush_ready(disp);
}


static void lvgl_touch_read_cb(lv_indev_t *indev, lv_indev_data_t *data) {
    esp_lcd_touch_handle_t touch = (esp_lcd_touch_handle_t)lv_indev_get_user_data(indev);
    esp_lcd_touch_point_data_t pt;
    uint8_t count = 0;

    esp_lcd_touch_read_data(touch);
    esp_err_t err = esp_lcd_touch_get_data(touch, &pt, &count, 1);
    if (err == ESP_OK && count > 0) {
        if (!s_backlight_on) {
            ESP_LOGI("BSP", "Touch detected while display is dark. Turning backlight ON!");
            bsp_backlight_set(100);
            lv_display_trigger_activity(NULL);
            data->state = LV_INDEV_STATE_RELEASED;
            return;
        }
        data->point.x = pt.x;
        data->point.y = pt.y;
        data->state = LV_INDEV_STATE_PRESSED;
        ESP_LOGI(TAG, "Touch point: (%d, %d)", (int)pt.x, (int)pt.y);
    } else {
        data->state = LV_INDEV_STATE_RELEASED;
    }
}

void bsp_backlight_set(uint8_t percent) {
    gpio_config_t bl_conf = {
        .mode = GPIO_MODE_OUTPUT,
        .pin_bit_mask = (1ULL << LCD_IO_BL),
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .pull_up_en = GPIO_PULLUP_DISABLE,
    };
    gpio_config(&bl_conf);
    s_backlight_on = (percent > 0);
    gpio_set_level((gpio_num_t)LCD_IO_BL, s_backlight_on ? 1 : 0);
}

// Initialization sequence specifically tailored for Guition ESP32-S3-4848S040 (ST7701S)
// Essential: 0xCD must be set to 0x00 to disable MDT (Memory Data Transfer) mode,
// preventing RGB565 sub-pixel scrambling / vertical stripes.
static const st7701_lcd_init_cmd_t s_st7701_4848_init_cmds[] = {
    // CMD2 BK0
    {0xFF, (uint8_t []){0x77, 0x01, 0x00, 0x00, 0x10}, 5, 0},
    {0xC0, (uint8_t []){0x3B, 0x00}, 2, 0},
    {0xC1, (uint8_t []){0x0D, 0x02}, 2, 0},
    {0xC2, (uint8_t []){0x31, 0x05}, 2, 0},
    {0xCD, (uint8_t []){0x00}, 1, 0}, // Disable MDT flag (critical: fixes vertical stripes!)

    // Positive Voltage Gamma Control
    {0xB0, (uint8_t []){0x00, 0x11, 0x18, 0x0E, 0x11, 0x06, 0x07, 0x08, 0x07, 0x22, 0x04, 0x12, 0x0F, 0xAA, 0x31, 0x18}, 16, 0},
    // Negative Voltage Gamma Control
    {0xB1, (uint8_t []){0x00, 0x11, 0x19, 0x0E, 0x12, 0x07, 0x08, 0x08, 0x08, 0x22, 0x04, 0x11, 0x11, 0xA9, 0x32, 0x18}, 16, 0},

    // CMD2 BK1 (Page 1)
    {0xFF, (uint8_t []){0x77, 0x01, 0x00, 0x00, 0x11}, 5, 0},
    {0xB0, (uint8_t []){0x60}, 1, 0},
    {0xB1, (uint8_t []){0x32}, 1, 0},
    {0xB2, (uint8_t []){0x07}, 1, 0},
    {0xB3, (uint8_t []){0x80}, 1, 0},
    {0xB5, (uint8_t []){0x49}, 1, 0},
    {0xB7, (uint8_t []){0x85}, 1, 0},
    {0xB8, (uint8_t []){0x21}, 1, 0},
    {0xC1, (uint8_t []){0x78}, 1, 0},
    {0xC2, (uint8_t []){0x78}, 1, 0},
    {0xE0, (uint8_t []){0x00, 0x1B, 0x02}, 3, 0},
    {0xE1, (uint8_t []){0x08, 0xA0, 0x00, 0x00, 0x07, 0xA0, 0x00, 0x00, 0x00, 0x44, 0x44}, 11, 0},
    {0xE2, (uint8_t []){0x11, 0x11, 0x44, 0x44, 0xED, 0xA0, 0x00, 0x00, 0xEC, 0xA0, 0x00, 0x00}, 12, 0},
    {0xE3, (uint8_t []){0x00, 0x00, 0x11, 0x11}, 4, 0},
    {0xE4, (uint8_t []){0x44, 0x44}, 2, 0},
    {0xE5, (uint8_t []){0x0A, 0xE9, 0xD8, 0xA0, 0x0C, 0xEB, 0xD8, 0xA0, 0x0E, 0xED, 0xD8, 0xA0, 0x10, 0xEF, 0xD8, 0xA0}, 16, 0},
    {0xE6, (uint8_t []){0x00, 0x00, 0x11, 0x11}, 4, 0},
    {0xE7, (uint8_t []){0x44, 0x44}, 2, 0},
    {0xE8, (uint8_t []){0x09, 0xE8, 0xD8, 0xA0, 0x0B, 0xEA, 0xD8, 0xA0, 0x0D, 0xEC, 0xD8, 0xA0, 0x0F, 0xEE, 0xD8, 0xA0}, 16, 0},
    {0xEB, (uint8_t []){0x02, 0x00, 0xE4, 0xE4, 0x88, 0x00, 0x40}, 7, 0},
    {0xEC, (uint8_t []){0x3C, 0x00}, 2, 0},
    {0xED, (uint8_t []){0xAB, 0x89, 0x76, 0x54, 0x02, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x20, 0x45, 0x67, 0x98, 0xBA}, 16, 0},

    // CMD2 BK3 (Page 3)
    {0xFF, (uint8_t []){0x77, 0x01, 0x00, 0x00, 0x13}, 5, 0},
    {0xE5, (uint8_t []){0xE4}, 1, 0},

    // Disable CMD2
    {0xFF, (uint8_t []){0x77, 0x01, 0x00, 0x00, 0x00}, 5, 0},

    // Sleep Out & Display On
    {0x11, (uint8_t []){0x00}, 0, 120},
    {0x29, (uint8_t []){0x00}, 0, 0},
};

lv_display_t *bsp_display_init(void) {
    ESP_LOGI(TAG, "Initializing ST7701S RGB panel for ESP32-S3-4848S040...");

    // 1. Install 3-wire SPI panel IO for ST7701S command initialization
    spi_line_config_t line_config = {
        .cs_io_type = IO_TYPE_GPIO,
        .cs_gpio_num = LCD_IO_SPI_CS,
        .scl_io_type = IO_TYPE_GPIO,
        .scl_gpio_num = LCD_IO_SPI_SCLK,
        .sda_io_type = IO_TYPE_GPIO,
        .sda_gpio_num = LCD_IO_SPI_SDA,
        .io_expander = NULL,
    };
    esp_lcd_panel_io_3wire_spi_config_t io_config = ST7701_PANEL_IO_3WIRE_SPI_CONFIG(line_config, 0);
    esp_lcd_panel_io_handle_t io_handle = NULL;
    ESP_ERROR_CHECK(esp_lcd_new_panel_io_3wire_spi(&io_config, &io_handle));

    // 2. Configure RGB interface and timings (12 MHz PCLK, 480x480)
    esp_lcd_rgb_panel_config_t rgb_config = {
        .clk_src = LCD_CLK_SRC_DEFAULT,
        .psram_trans_align = 64,
        .data_width = 16,
        .bits_per_pixel = 16,
        .de_gpio_num = LCD_IO_RGB_DE,
        .pclk_gpio_num = LCD_IO_RGB_PCLK,
        .vsync_gpio_num = LCD_IO_RGB_VSYNC,
        .hsync_gpio_num = LCD_IO_RGB_HSYNC,
        .disp_gpio_num = -1,
        .data_gpio_nums = {
            LCD_IO_RGB_DATA0, LCD_IO_RGB_DATA1, LCD_IO_RGB_DATA2, LCD_IO_RGB_DATA3,
            LCD_IO_RGB_DATA4, LCD_IO_RGB_DATA5, LCD_IO_RGB_DATA6, LCD_IO_RGB_DATA7,
            LCD_IO_RGB_DATA8, LCD_IO_RGB_DATA9, LCD_IO_RGB_DATA10, LCD_IO_RGB_DATA11,
            LCD_IO_RGB_DATA12, LCD_IO_RGB_DATA13, LCD_IO_RGB_DATA14, LCD_IO_RGB_DATA15,
        },
        .timings = {
            .pclk_hz = 12 * 1000 * 1000,
            .h_res = BSP_LCD_H_RES,
            .v_res = BSP_LCD_V_RES,
            .hsync_pulse_width = 8,
            .hsync_back_porch = 20,
            .hsync_front_porch = 10,
            .vsync_pulse_width = 8,
            .vsync_back_porch = 10,
            .vsync_front_porch = 10,
            .flags = {
                .pclk_active_neg = false,
            },
        },
        .flags = {
            .fb_in_psram = true,
            .double_fb = false,
        },
    };

    st7701_vendor_config_t vendor_config = {
        .init_cmds = s_st7701_4848_init_cmds,
        .init_cmds_size = sizeof(s_st7701_4848_init_cmds) / sizeof(s_st7701_4848_init_cmds[0]),
        .rgb_config = &rgb_config,
        .flags = {
            .use_mipi_interface = 0,
            .mirror_by_cmd = 0,
            .enable_io_multiplex = 0,
        },
    };

    esp_lcd_panel_dev_config_t panel_dev_config = {
        .reset_gpio_num = -1,
        .rgb_ele_order = LCD_RGB_ELEMENT_ORDER_BGR,
        .bits_per_pixel = 16,
        .vendor_config = &vendor_config,
    };

    ESP_ERROR_CHECK(esp_lcd_new_panel_st7701(io_handle, &panel_dev_config, &s_panel_handle));
    ESP_ERROR_CHECK(esp_lcd_panel_reset(s_panel_handle));
    ESP_ERROR_CHECK(esp_lcd_panel_init(s_panel_handle));
    ESP_ERROR_CHECK(esp_lcd_panel_disp_on_off(s_panel_handle, true));

    // Turn on backlight
    bsp_backlight_set(100);

    // 3. Register with LVGL v9
    lv_display_t *disp = lv_display_create(BSP_LCD_H_RES, BSP_LCD_V_RES);
    lv_display_set_user_data(disp, s_panel_handle);
    lv_display_set_color_format(disp, LV_COLOR_FORMAT_RGB565);
    lv_display_set_flush_cb(disp, lvgl_flush_cb);

    // Allocate draw buffers in PSRAM (1/10th screen height per buffer)
    size_t buf_size = BSP_LCD_H_RES * (BSP_LCD_V_RES / 10) * sizeof(lv_color16_t);
    void *buf1 = heap_caps_malloc(buf_size, MALLOC_CAP_SPIRAM);
    void *buf2 = heap_caps_malloc(buf_size, MALLOC_CAP_SPIRAM);
    assert(buf1 != NULL);

    lv_display_set_buffers(disp, buf1, buf2, buf_size, LV_DISPLAY_RENDER_MODE_PARTIAL);
    ESP_LOGI(TAG, "ST7701S display initialized successfully (%dx%d)", BSP_LCD_H_RES, BSP_LCD_V_RES);
    return disp;
}

lv_indev_t *bsp_touch_init(lv_display_t *disp) {
    ESP_LOGI(TAG, "Initializing Goodix GT911 touch panel on I2C...");

    i2c_config_t i2c_conf = {
        .mode = I2C_MODE_MASTER,
        .sda_io_num = TOUCH_I2C_SDA,
        .scl_io_num = TOUCH_I2C_SCL,
        .sda_pullup_en = GPIO_PULLUP_ENABLE,
        .scl_pullup_en = GPIO_PULLUP_ENABLE,
        .master.clk_speed = 400 * 1000,
    };
    ESP_ERROR_CHECK(i2c_param_config(TOUCH_I2C_NUM, &i2c_conf));
    ESP_ERROR_CHECK(i2c_driver_install(TOUCH_I2C_NUM, i2c_conf.mode, 0, 0, 0));

    esp_lcd_panel_io_handle_t tp_io_handle = NULL;
    esp_lcd_panel_io_i2c_config_t tp_io_config = ESP_LCD_TOUCH_IO_I2C_GT911_CONFIG();
    tp_io_config.scl_speed_hz = 0; // Required by ESP-IDF v5.5 legacy I2C panel IO driver
    ESP_ERROR_CHECK(esp_lcd_new_panel_io_i2c((esp_lcd_i2c_bus_handle_t)TOUCH_I2C_NUM, &tp_io_config, &tp_io_handle));

    esp_lcd_touch_config_t tp_cfg = {
        .x_max = BSP_LCD_H_RES,
        .y_max = BSP_LCD_V_RES,
        .rst_gpio_num = -1,
        .int_gpio_num = -1,
        .flags = {
            .swap_xy = 1,
            .mirror_x = 1,
            .mirror_y = 0,
        },
    };
    ESP_ERROR_CHECK(esp_lcd_touch_new_i2c_gt911(tp_io_handle, &tp_cfg, &s_touch_handle));

    // Register LVGL indev
    lv_indev_t *indev = lv_indev_create();
    lv_indev_set_type(indev, LV_INDEV_TYPE_POINTER);
    lv_indev_set_user_data(indev, s_touch_handle);
    lv_indev_set_read_cb(indev, lvgl_touch_read_cb);
    if (disp) {
        lv_indev_set_display(indev, disp);
    }

    ESP_LOGI(TAG, "GT911 touch panel initialized successfully");
    return indev;
}

#else // !ESP_PLATFORM (Host simulation / local tests)

void bsp_backlight_set(uint8_t percent) {
    s_backlight_on = (percent > 0);
}

lv_display_t *bsp_display_init(void) {
    return NULL;
}

lv_indev_t *bsp_touch_init(lv_display_t *disp) {
    (void)disp;
    return NULL;
}

#endif // ESP_PLATFORM

static const esp_lvgl_bsp_ops_t s_guition_s3_4848s040_ops = {
    .board_name      = "Guition ESP32-S3-4848S040 (ST7701S RGB + GT911)",
    .init_display    = bsp_display_init,
    .init_touch      = bsp_touch_init,
    .set_backlight   = bsp_backlight_set,
    .backlight_is_on = bsp_backlight_is_on,
};

const esp_lvgl_bsp_ops_t *bsp_4848s040_get_ops(void) {
    return &s_guition_s3_4848s040_ops;
}

