#ifndef BRIDGE_PROTOCOL_H
#define BRIDGE_PROTOCOL_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    PARAM_NONE           = 0,
    PARAM_SETPOINT_HZ    = 1, // Slider: i32_val setpoint
    PARAM_SYSTEM_ENABLE  = 2, // Switch: bool_val enable/run state
    PARAM_TRIGGER_RESET  = 3, // Push Button: momentary command trigger
    PARAM_CYCLE_COUNT    = 4, // Readout: u32_val telemetry counter
    PARAM_ARC_KNOB       = 5, // Arc: i32_val knob setpoint
    PARAM_PROGRESS_BAR   = 6, // Bar: i32_val level indicator
    PARAM_AUX_CHECKBOX   = 7, // Checkbox: bool_val check state
    PARAM_STATUS_LED     = 8, // LED: bool_val status
    PARAM_BUSY_SPINNER   = 9, // Spinner: bool_val visibility
    PARAM_MODE_DROPDOWN  = 10,// Dropdown: i32_val option index
    PARAM_SPEED_ROLLER   = 11,// Roller: i32_val option index
    PARAM_PRECISION_STEP = 12,// Spinbox: i32_val step value
    // Tier 2: Text, Keypads, Modals & Gauges
    PARAM_TEXT_INPUT     = 13,// TextArea: str_val buffer
    PARAM_KEYBOARD       = 14,// Keyboard: i32_val action (1=ready, 2=cancel)
    PARAM_KEYPAD_MATRIX  = 15,// ButtonMatrix: i32_val index, str_val key
    PARAM_ALERT_MODAL    = 16,// MsgBox: i32_val btn, bool_val open/close
    PARAM_GAUGE_SCALE    = 17,// Scale: i32_val needle value
    // Tier 3: Containers, Navigation & Data Streams
    PARAM_NAV_TAB        = 18,// TabView: i32_val active tab
    PARAM_NAV_TILE       = 19,// TileView: i32_val active tile
    PARAM_MENU_NAV       = 20,// Menu: i32_val selected item
    PARAM_ITEM_LIST      = 21,// List: i32_val selected item
    PARAM_TELEMETRY_CHART= 22,// Chart: i32_val streaming sample
    PARAM_DATA_TABLE     = 23,// Table: i32_val/str_val
    PARAM_CANVAS_DRAW    = 25,// Canvas
    PARAM_TEXT_AREA      = 26,// TextArea (multiline): str_val buffer
} param_id_t;


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
