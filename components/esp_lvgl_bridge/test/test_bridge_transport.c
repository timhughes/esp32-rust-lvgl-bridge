#include "unity.h"
#include "bridge_protocol.h"
#include "esp_lvgl_bridge.h"

TEST_CASE("bridge_transport init and command queue send/receive", "[esp_lvgl_bridge]") {
    bridge_transport_init(8, 8);

    ui_command_t send_cmd = {
        .param_id = 42,
        .seq_id = 100,
        .value.i32_val = 1234,
    };

    TEST_ASSERT_TRUE(bridge_send_command_from_ui(&send_cmd));

    ui_command_t recv_cmd = {0};
    TEST_ASSERT_TRUE(bridge_recv_command(&recv_cmd, 10));
    TEST_ASSERT_EQUAL_INT(42, recv_cmd.param_id);
    TEST_ASSERT_EQUAL_UINT32(100, recv_cmd.seq_id);
    TEST_ASSERT_EQUAL_INT32(1234, recv_cmd.value.i32_val);

    // Queue should now be empty (non-blocking)
    TEST_ASSERT_FALSE(bridge_recv_command(&recv_cmd, 0));
}

TEST_CASE("bridge_transport telemetry queue send and receive", "[esp_lvgl_bridge]") {
    bridge_transport_init(8, 8);

    device_telemetry_t send_telem = {
        .param_id = 10,
        .ack_seq_id = 55,
        .value.bool_val = true,
    };

    TEST_ASSERT_TRUE(bridge_send_telemetry(&send_telem));

    device_telemetry_t recv_telem = {0};
    TEST_ASSERT_TRUE(bridge_recv_telemetry_in_ui(&recv_telem));
    TEST_ASSERT_EQUAL_INT(10, recv_telem.param_id);
    TEST_ASSERT_EQUAL_UINT32(55, recv_telem.ack_seq_id);
    TEST_ASSERT_TRUE(recv_telem.value.bool_val);

    // Queue should now be empty
    TEST_ASSERT_FALSE(bridge_recv_telemetry_in_ui(&recv_telem));
}

TEST_CASE("bridge_transport null pointer guard checks", "[esp_lvgl_bridge]") {
    bridge_transport_init(8, 8);
    TEST_ASSERT_FALSE(bridge_send_command_from_ui(NULL));
    TEST_ASSERT_FALSE(bridge_recv_command(NULL, 0));
    TEST_ASSERT_FALSE(bridge_send_telemetry(NULL));
    TEST_ASSERT_FALSE(bridge_recv_telemetry_in_ui(NULL));
}
