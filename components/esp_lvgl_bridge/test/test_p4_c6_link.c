#include "unity.h"
#include "p4_c6_link.h"
#include <string.h>

static uint8_t s_rx_buf[128];
static uint16_t s_rx_len = 0;
static p4_c6_channel_t s_rx_chan = 0;

static void test_rx_cb(p4_c6_channel_t chan, const uint8_t *data, uint16_t len, void *user_data) {
    (void)user_data;
    s_rx_chan = chan;
    s_rx_len = len;
    if (len <= sizeof(s_rx_buf)) {
        memcpy(s_rx_buf, data, len);
    }
}

TEST_CASE("p4_c6_link loopback transmission and callback dispatch", "[esp_lvgl_bridge]") {
    p4_c6_link_config_t cfg = {
        .bus_type = P4_C6_BUS_LOOPBACK_MOCK,
    };
    TEST_ASSERT_EQUAL(ESP_OK, p4_c6_link_init(&cfg));
    TEST_ASSERT_TRUE(p4_c6_link_is_alive());

    TEST_ASSERT_EQUAL(ESP_OK, p4_c6_link_register_rx_callback(P4_C6_CHAN_TELEMETRY, test_rx_cb, NULL));

    s_rx_len = 0;
    const char *payload = "PING_TELEMETRY_PACKET";
    TEST_ASSERT_EQUAL(ESP_OK, p4_c6_link_send(P4_C6_CHAN_TELEMETRY, payload, (uint16_t)strlen(payload), 50));

    TEST_ASSERT_EQUAL(P4_C6_CHAN_TELEMETRY, s_rx_chan);
    TEST_ASSERT_EQUAL_UINT16(strlen(payload), s_rx_len);
    TEST_ASSERT_EQUAL_STRING_LEN(payload, (char *)s_rx_buf, s_rx_len);

    p4_c6_link_stats_t stats = {0};
    p4_c6_link_get_stats(&stats);
    TEST_ASSERT_GREATER_THAN_UINT32(0, stats.tx_packets);
    TEST_ASSERT_GREATER_THAN_UINT32(0, stats.rx_packets);
}

TEST_CASE("p4_c6_link invalid arguments handling", "[esp_lvgl_bridge]") {
    TEST_ASSERT_EQUAL(ESP_ERR_INVALID_ARG, p4_c6_link_init(NULL));
    TEST_ASSERT_EQUAL(ESP_ERR_INVALID_ARG, p4_c6_link_register_rx_callback(P4_C6_CHAN_MAX, test_rx_cb, NULL));
    TEST_ASSERT_EQUAL(ESP_ERR_INVALID_ARG, p4_c6_link_send(P4_C6_CHAN_MAX, "TEST", 4, 10));
    TEST_ASSERT_EQUAL(ESP_ERR_INVALID_ARG, p4_c6_link_send(P4_C6_CHAN_SYSTEM, "TEST", P4_C6_MAX_PAYLOAD_LEN + 1, 10));
}
