#include "p4_c6_link.h"
#include <string.h>

#ifdef ESP_PLATFORM
#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "driver/gpio.h"
#define TAG "P4_C6_LINK"
#else
#include <stdio.h>
#define TAG "P4_C6_LINK"
#define ESP_LOGI(tag, fmt, ...) printf("[%s] " fmt "\n", tag, ##__VA_ARGS__)
#define ESP_LOGE(tag, fmt, ...) fprintf(stderr, "[%s ERROR] " fmt "\n", tag, ##__VA_ARGS__)
#endif

typedef struct {
    p4_c6_rx_cb_t cb;
    void *user_data;
} channel_handler_t;

static p4_c6_link_config_t s_config;
static p4_c6_link_stats_t s_stats;
static channel_handler_t s_handlers[P4_C6_CHAN_MAX];
static uint16_t s_tx_seq = 0;
static bool s_is_initialized = false;

// Fast CRC-CCITT (polynomial 0x1021)
static uint16_t compute_crc16(const uint8_t *data, uint16_t len) {
    uint16_t crc = 0xFFFF;
    for (uint16_t i = 0; i < len; i++) {
        crc ^= (uint16_t)data[i] << 8;
        for (uint8_t bit = 0; bit < 8; bit++) {
            if (crc & 0x8000) {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    return crc;
}

esp_err_t p4_c6_link_init(const p4_c6_link_config_t *config) {
    if (!config) return ESP_ERR_INVALID_ARG;

    s_config = *config;
    memset(&s_stats, 0, sizeof(s_stats));
    memset(s_handlers, 0, sizeof(s_handlers));
    s_tx_seq = 0;
    s_is_initialized = true;

    ESP_LOGI(TAG, "Initialized P4-C6 link in %s mode",
             config->bus_type == P4_C6_BUS_SDIO_3_0 ? "SDIO 3.0" :
             config->bus_type == P4_C6_BUS_SPI_HS ? "SPI HS" : "LOOPBACK MOCK");

    return ESP_OK;
}

esp_err_t p4_c6_link_register_rx_callback(p4_c6_channel_t channel, p4_c6_rx_cb_t cb, void *user_data) {
    if (channel >= P4_C6_CHAN_MAX) return ESP_ERR_INVALID_ARG;
    s_handlers[channel].cb = cb;
    s_handlers[channel].user_data = user_data;
    return ESP_OK;
}

esp_err_t p4_c6_link_send(p4_c6_channel_t channel, const void *data, uint16_t len, uint32_t timeout_ms) {
    if (!s_is_initialized) return ESP_FAIL;
    if (channel >= P4_C6_CHAN_MAX || len > P4_C6_MAX_PAYLOAD_LEN) return ESP_ERR_INVALID_ARG;

    p4_c6_frame_header_t hdr = {
        .magic = P4_C6_LINK_MAGIC,
        .channel = (uint8_t)channel,
        .flags = 0,
        .seq_num = ++s_tx_seq,
        .payload_len = len,
        .crc16 = compute_crc16((const uint8_t *)data, len),
    };

    s_stats.tx_packets++;
    s_stats.tx_bytes += (sizeof(hdr) + len);

    // If loopback or mock mode, immediately dispatch to RX handler
    if (s_config.bus_type == P4_C6_BUS_LOOPBACK_MOCK) {
        if (s_handlers[channel].cb) {
            s_handlers[channel].cb(channel, (const uint8_t *)data, len, s_handlers[channel].user_data);
            s_stats.rx_packets++;
            s_stats.rx_bytes += (sizeof(hdr) + len);
        }
        return ESP_OK;
    }

    // In hardware mode (SDIO/SPI), write to DMA descriptor and pulse handshake pin
    // Placeholder for target hardware DMA write
    (void)timeout_ms;
    return ESP_OK;
}

void p4_c6_link_get_stats(p4_c6_link_stats_t *out_stats) {
    if (out_stats) *out_stats = s_stats;
}

bool p4_c6_link_is_alive(void) {
    return s_is_initialized;
}
