#ifndef BRIDGE_PROTOCOL_H
#define BRIDGE_PROTOCOL_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    PARAM_NONE           = 0,  // Sentinel: Unbound widget or root container without parent
    PARAM_WIDGET_FLAGS   = 36, // Framework protocol: ack_seq_id=target_param_id, u32_val=flags
} param_id_t;

typedef enum {
    WIDGET_FLAG_NONE        = 0,
    WIDGET_FLAG_HIDDEN      = (1 << 0),
    WIDGET_FLAG_VISIBLE     = (1 << 1),
    WIDGET_FLAG_DISABLED    = (1 << 2),
    WIDGET_FLAG_ENABLED     = (1 << 3),
    WIDGET_FLAG_CLICKABLE   = (1 << 4),
    WIDGET_FLAG_UNCLICKABLE = (1 << 5),
} widget_flag_op_t;

typedef enum {
    LAYOUT_FLOW_NONE           = 0,
    LAYOUT_FLOW_ROW            = 1,
    LAYOUT_FLOW_COLUMN         = 2,
    LAYOUT_FLOW_ROW_WRAP       = 3,
    LAYOUT_FLOW_ROW_REVERSE    = 4,
    LAYOUT_FLOW_COLUMN_WRAP    = 5,
    LAYOUT_FLOW_COLUMN_REVERSE = 6,
} layout_flow_t;


typedef union {
    int32_t  i32_val;
    uint32_t u32_val;
    float    f32_val;
    bool     bool_val;
    char     str_val[32];
} param_val_t;

// UI -> Rust Worker (Command Queue)
typedef struct {
    param_id_t  param_id;
    uint32_t    seq_id;
    param_val_t value;
} ui_command_t;

// Rust Worker -> UI (Telemetry Queue)
typedef struct {
    param_id_t  param_id;
    uint32_t    ack_seq_id;
    param_val_t value;
} device_telemetry_t;

// Transport Initialization
void bridge_transport_init(size_t cmd_depth, size_t telem_depth);

// UI Transport Functions (called from C UI)
bool bridge_send_command_from_ui(const ui_command_t *cmd);
bool bridge_recv_telemetry_in_ui(device_telemetry_t *out_telem);

// Rust Worker Transport FFI (called from Rust staticlib)
bool bridge_recv_command(ui_command_t *out_cmd, uint32_t timeout_ms);
bool bridge_send_telemetry(const device_telemetry_t *telem);

// Declarative UI Schema Types
typedef enum {
    WIDGET_TYPE_SWITCH        = 1,
    WIDGET_TYPE_SLIDER        = 2,
    WIDGET_TYPE_BUTTON        = 3,
    WIDGET_TYPE_LABEL         = 4,
    WIDGET_TYPE_ARC           = 5,
    WIDGET_TYPE_BAR           = 6,
    WIDGET_TYPE_CHECKBOX      = 7,
    WIDGET_TYPE_LED           = 8,
    WIDGET_TYPE_SPINNER       = 9,
    WIDGET_TYPE_DROPDOWN      = 10,
    WIDGET_TYPE_ROLLER        = 11,
    WIDGET_TYPE_SPINBOX       = 12,
    // Tier 2
    WIDGET_TYPE_TEXTAREA      = 13,
    WIDGET_TYPE_KEYBOARD      = 14,
    WIDGET_TYPE_BUTTON_MATRIX = 15,
    WIDGET_TYPE_MSGBOX        = 16,
    WIDGET_TYPE_SCALE         = 17,
    // Tier 3
    WIDGET_TYPE_TABVIEW       = 18,
    WIDGET_TYPE_TILEVIEW      = 19,
    WIDGET_TYPE_MENU          = 20,
    WIDGET_TYPE_LIST          = 21,
    WIDGET_TYPE_CHART         = 22,
    WIDGET_TYPE_TABLE         = 23,
    WIDGET_TYPE_CALENDAR      = 24,
    WIDGET_TYPE_CANVAS        = 25,
    // Phase 1: Visuals & IoT Core
    WIDGET_TYPE_IMAGE         = 26,
    WIDGET_TYPE_LINE          = 27,
    WIDGET_TYPE_ARCLABEL      = 28,
    WIDGET_TYPE_QRCODE        = 29,
    // Phase 2: Complex UI & Rich Media
    WIDGET_TYPE_WIN           = 30,
    WIDGET_TYPE_IMAGEBUTTON   = 31,
    WIDGET_TYPE_SPAN_GROUP    = 32,
    WIDGET_TYPE_ANIMIMAGE     = 33,
    WIDGET_TYPE_BARCODE       = 34,
} widget_type_t;

typedef enum {
    UI_ALIGN_TOP_LEFT     = 1,
    UI_ALIGN_TOP_MID      = 2,
    UI_ALIGN_TOP_RIGHT    = 3,
    UI_ALIGN_CENTER       = 4,
    UI_ALIGN_BOTTOM_LEFT  = 5,
    UI_ALIGN_BOTTOM_MID   = 6,
    UI_ALIGN_BOTTOM_RIGHT = 7,
} ui_align_t;

typedef struct {
    widget_type_t widget_type;
    param_id_t    param_id;
    ui_align_t    align;
    int32_t       offset_x;
    int32_t       offset_y;
    int32_t       min_val;
    int32_t       max_val;
    int32_t       initial_val;
    uint32_t      holdoff_ms;
    const char   *label_primary;
    const char   *label_secondary;
    param_id_t    parent_param_id;
    int32_t       width;
    int32_t       height;
    uint32_t      tab_index;
    layout_flow_t layout_flow;
} widget_descriptor_t;

// Rust Worker Lifecycle Entry Points (implemented in Rust staticlib)
void rust_worker_init(void);
void rust_worker_step(void);
const widget_descriptor_t *rust_get_minimal_schema(size_t *out_count);
const widget_descriptor_t *rust_get_everything_schema(size_t *out_count);

#ifdef __cplusplus
}
#endif

#endif // BRIDGE_PROTOCOL_H
