#include "bridge_protocol.h"

#ifdef ESP_PLATFORM
#include "freertos/FreeRTOS.h"
#include "esp_log.h"

static QueueHandle_t s_cmd_queue = NULL;
static QueueHandle_t s_telem_queue = NULL;
static bridge_transport_stats_t s_stats = {0};

void bridge_transport_init(size_t cmd_depth, size_t telem_depth) {
    if (s_cmd_queue == NULL) {
        s_cmd_queue = xQueueCreate(cmd_depth, sizeof(ui_command_t));
    }
    if (s_telem_queue == NULL) {
        s_telem_queue = xQueueCreate(telem_depth, sizeof(device_telemetry_t));
    }
}

void bridge_transport_get_stats(bridge_transport_stats_t *out_stats) {
    if (out_stats) {
        *out_stats = s_stats;
    }
}

bool bridge_send_command_from_ui(const ui_command_t *cmd) {
    if (!s_cmd_queue || !cmd) return false;
    ESP_LOGI("BRIDGE", "UI event -> command param=%lu, seq=%lu", (unsigned long)cmd->param_id, (unsigned long)cmd->seq_id);
    if (xQueueSend(s_cmd_queue, cmd, 0) == pdTRUE) {
        s_stats.cmd_sent++;
        return true;
    }
    s_stats.cmd_dropped++;
    ESP_LOGW("BRIDGE", "Command queue saturated! Dropped cmd param=%lu, seq=%lu (total dropped: %lu)",
             (unsigned long)cmd->param_id, (unsigned long)cmd->seq_id, (unsigned long)s_stats.cmd_dropped);
    return false;
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
    if (xQueueSend(s_telem_queue, telem, 0) == pdTRUE) {
        s_stats.telem_sent++;
        return true;
    }
    s_stats.telem_dropped++;
    ESP_LOGW("BRIDGE", "Telemetry queue saturated! Dropped telem param=%lu, ack_seq=%lu (total dropped: %lu)",
             (unsigned long)telem->param_id, (unsigned long)telem->ack_seq_id, (unsigned long)s_stats.telem_dropped);
    return false;
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

static bridge_transport_stats_t s_stats = {0};

void host_transport_reset(void) {
    s_cmd_head = 0;
    s_cmd_tail = 0;
    s_cmd_count = 0;
    s_telem_head = 0;
    s_telem_tail = 0;
    s_telem_count = 0;
    memset(&s_stats, 0, sizeof(s_stats));
}

size_t host_transport_get_cmd_count(void) {
    return s_cmd_count;
}

size_t host_transport_get_telem_count(void) {
    return s_telem_count;
}

void bridge_transport_get_stats(bridge_transport_stats_t *out_stats) {
    if (out_stats) {
        *out_stats = s_stats;
    }
}

void bridge_transport_init(size_t cmd_depth, size_t telem_depth) {
    (void)cmd_depth;
    (void)telem_depth;
    host_transport_reset();
}

bool bridge_send_command_from_ui(const ui_command_t *cmd) {
    if (!cmd) return false;
    if (s_cmd_count >= MAX_QUEUE_DEPTH) {
        s_stats.cmd_dropped++;
        return false;
    }
    s_cmd_queue[s_cmd_head] = *cmd;
    s_cmd_head = (s_cmd_head + 1) % MAX_QUEUE_DEPTH;
    s_cmd_count++;
    s_stats.cmd_sent++;
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
    if (!telem) return false;
    if (s_telem_count >= MAX_QUEUE_DEPTH) {
        s_stats.telem_dropped++;
        return false;
    }
    s_telem_queue[s_telem_head] = *telem;
    s_telem_head = (s_telem_head + 1) % MAX_QUEUE_DEPTH;
    s_telem_count++;
    s_stats.telem_sent++;
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
