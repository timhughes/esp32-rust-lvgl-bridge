#ifndef P4_C6_LINK_H
#define P4_C6_LINK_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef ESP_PLATFORM
#include "esp_err.h"
#else
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_FAIL -1
#define ESP_ERR_TIMEOUT -2
#define ESP_ERR_INVALID_ARG -3
#define ESP_ERR_NO_MEM -4
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define P4_C6_LINK_MAGIC 0xE5P4
#define P4_C6_MAX_PAYLOAD_LEN 1536

typedef enum {
    P4_C6_CHAN_SYSTEM    = 0x01, // Heartbeat, sync, coprocessor power control
    P4_C6_CHAN_WIFI      = 0x02, // 802.3 Ethernet frames / ESP-Hosted WiFi
    P4_C6_CHAN_BLE       = 0x03, // HCI commands/events for BLE 5.3
    P4_C6_CHAN_802154    = 0x04, // Thread / Zigbee / Matter 802.15.4 frames
    P4_C6_CHAN_TELEMETRY = 0x05, // Co-processor health and RF metrics
    P4_C6_CHAN_MAX
} p4_c6_channel_t;

typedef enum {
    P4_C6_BUS_SDIO_3_0 = 0,
    P4_C6_BUS_SPI_HS,
    P4_C6_BUS_LOOPBACK_MOCK
} p4_c6_bus_type_t;

#pragma pack(push, 1)
typedef struct {
    uint16_t magic;          // 0xE5P4
    uint8_t  channel;        // p4_c6_channel_t
    uint8_t  flags;          // Bit 0: ACK requested, Bit 1: Is ACK, Bit 2: Fragment
    uint16_t seq_num;        // Monotonic sequence ID
    uint16_t payload_len;    // Length of payload (<= 1536)
    uint16_t crc16;          // CRC-CCITT payload checksum
} p4_c6_frame_header_t;
#pragma pack(pop)

typedef void (*p4_c6_rx_cb_t)(p4_c6_channel_t channel, const uint8_t *data, uint16_t len, void *user_data);

typedef struct {
    p4_c6_bus_type_t bus_type;
    int pin_clk;
    int pin_cmd;
    int pin_d0;
    int pin_d1;
    int pin_d2;
    int pin_d3;
    int pin_handshake_in;    // Slave Data Ready interrupt
    int pin_handshake_out;   // Host Data Ready line
    uint32_t bus_freq_khz;   // e.g. 50000 (50MHz) or 200000 (200MHz DDR50)
} p4_c6_link_config_t;

typedef struct {
    uint32_t tx_packets;
    uint32_t rx_packets;
    uint32_t tx_bytes;
    uint32_t rx_bytes;
    uint32_t crc_errors;
    uint32_t dropped_packets;
} p4_c6_link_stats_t;

/**
 * @brief Initialize the ESP32-P4 <-> ESP32-C6 communication link.
 */
esp_err_t p4_c6_link_init(const p4_c6_link_config_t *config);

/**
 * @brief Transmit a packet over the specified channel.
 */
esp_err_t p4_c6_link_send(p4_c6_channel_t channel, const void *data, uint16_t len, uint32_t timeout_ms);

/**
 * @brief Register a callback for incoming channel packets.
 */
esp_err_t p4_c6_link_register_rx_callback(p4_c6_channel_t channel, p4_c6_rx_cb_t cb, void *user_data);

/**
 * @brief Retrieve runtime link statistics.
 */
void p4_c6_link_get_stats(p4_c6_link_stats_t *out_stats);

/**
 * @brief Check if link is healthy and synchronized.
 */
bool p4_c6_link_is_alive(void);

#ifdef __cplusplus
}
#endif

#endif // P4_C6_LINK_H
