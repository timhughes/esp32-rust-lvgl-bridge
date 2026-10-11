#include "bridge_protocol.h"

#ifdef ESP_PLATFORM
#include "freertos/FreeRTOS.h"
#include "esp_log.h"

static QueueHandle_t s_cmd_queue = NULL;
static QueueHandle_t s_telem_queue = NULL;

void bridge_transport_init(size_t cmd_depth, size_t telem_depth) {
    if (s_cmd_queue == NULL) {
        s_cmd_queue = xQueueCreate(cmd_depth, sizeof(ui_command_t));
    }
    if (s_telem_queue == NULL) {
        s_telem_queue = xQueueCreate(telem_depth, sizeof(device_telemetry_t));
    }
}

bool bridge_send_command_from_ui(const ui_command_t *cmd) {
    if (!s_cmd_queue || !cmd) return false;
    ESP_LOGI("BRIDGE", "UI event -> command param=%d, seq=%u", cmd->param_id, (unsigned int)cmd->seq_id);
    return xQueueSend(s_cmd_queue, cmd, 0) == pdTRUE;
}


bool bridge_recv_telemetry_in_ui(device_telemetry_t *out_telem) {
    if (!s_telem_queue || !out_telem) return false;
    return xQueueReceive(s_telem_queue, out_telem, 0) == pdTRUE;
}

bool bridge_recv_command(ui_command_t *out_cmd, uint32_t timeout_ms) {
    if (!s_cmd_queue || !out_cmd) return false;
    TickType_t ticks = (timeout_ms == 0) ? 0 : pdMS_TO_TICKS(timeout_ms);
    return xQueueReceive(s_cmd_queue, out_cmd, ticks) == pdTRUE;
}

bool bridge_send_telemetry(const device_telemetry_t *telem) {
    if (!s_telem_queue || !telem) return false;
    return xQueueSend(s_telem_queue, telem, 0) == pdTRUE;
}

#else // Host / Simulation mode

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

void bridge_transport_init(size_t cmd_depth, size_t telem_depth) {
    (void)cmd_depth;
    (void)telem_depth;
    host_transport_reset();
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

#endif // ESP_PLATFORM
