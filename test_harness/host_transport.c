#include "host_transport.h"
#include <string.h>

#define MAX_QUEUE_DEPTH 256

static ui_command_t s_cmd_queue[MAX_QUEUE_DEPTH];
static size_t s_cmd_head = 0;
static size_t s_cmd_tail = 0;
static size_t s_cmd_count = 0;

static device_telemetry_t s_telem_queue[MAX_QUEUE_DEPTH];
static size_t s_telem_head = 0;
static size_t s_telem_tail = 0;
static size_t s_telem_count = 0;

void bridge_transport_init(size_t cmd_depth, size_t telem_depth) {
    (void)cmd_depth;
    (void)telem_depth;
    host_transport_reset();
}

void host_transport_reset(void) {
    s_cmd_head = 0;
    s_cmd_tail = 0;
    s_cmd_count = 0;

    s_telem_head = 0;
    s_telem_tail = 0;
    s_telem_count = 0;
}

size_t host_transport_get_cmd_count(void) {
    return s_cmd_count;
}

size_t host_transport_get_telem_count(void) {
    return s_telem_count;
}

bool bridge_send_command_from_ui(const ui_command_t *cmd) {
    if (!cmd || s_cmd_count >= MAX_QUEUE_DEPTH) return false;
    s_cmd_queue[s_cmd_head] = *cmd;
    s_cmd_head = (s_cmd_head + 1) % MAX_QUEUE_DEPTH;
    s_cmd_count++;
    return true;
}

bool bridge_recv_command(ui_command_t *out_cmd, uint32_t timeout_ms) {
    (void)timeout_ms;
    if (!out_cmd || s_cmd_count == 0) return false;
    *out_cmd = s_cmd_queue[s_cmd_tail];
    s_cmd_tail = (s_cmd_tail + 1) % MAX_QUEUE_DEPTH;
    s_cmd_count--;
    return true;
}

bool bridge_send_telemetry(const device_telemetry_t *telem) {
    if (!telem || s_telem_count >= MAX_QUEUE_DEPTH) return false;
    s_telem_queue[s_telem_head] = *telem;
    s_telem_head = (s_telem_head + 1) % MAX_QUEUE_DEPTH;
    s_telem_count++;
    return true;
}

bool bridge_recv_telemetry_in_ui(device_telemetry_t *out_telem) {
    if (!out_telem || s_telem_count == 0) return false;
    *out_telem = s_telem_queue[s_telem_tail];
    s_telem_tail = (s_telem_tail + 1) % MAX_QUEUE_DEPTH;
    s_telem_count--;
    return true;
}
