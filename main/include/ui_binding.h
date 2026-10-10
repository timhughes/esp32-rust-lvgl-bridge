#ifndef UI_BINDING_H
#define UI_BINDING_H

#include <stdint.h>
#include <stdbool.h>
#include "lvgl.h"
#include "bridge_protocol.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DEFAULT_HOLDOFF_MS 500

typedef enum {
    ARB_TRACKING = 0,
    ARB_DRAGGING,
    ARB_HOLDOFF,
} ui_arb_state_t;

typedef struct {
    lv_obj_t *slider;
    lv_obj_t *label;
    param_id_t param_id;
    int32_t min;
    int32_t max;
    int32_t initial_val;
    const char *format_setting; // e.g. "%ld Hz (Setting)"
    const char *format_actual;  // e.g. "%ld Hz (Actual)"
    uint32_t holdoff_ms;        // 0 uses DEFAULT_HOLDOFF_MS
} ui_binding_slider_opts_t;

typedef struct {
    lv_obj_t *sw;
    lv_obj_t *label;
    param_id_t param_id;
    bool initial_state;
    const char *label_on;       // e.g. "System: RUNNING"
    const char *label_off;      // e.g. "System: STOPPED"
} ui_binding_switch_opts_t;

typedef struct {
    lv_obj_t *button;
    param_id_t param_id;
    uint32_t command_val;
} ui_binding_button_opts_t;

typedef struct {
    lv_obj_t *label;
    param_id_t param_id;
    const char *format;         // e.g. "Cycles: %lu"
} ui_binding_label_opts_t;

/**
 * @brief Initialize the declarative UI binding engine.
 */
void ui_engine_init(void);

/**
 * @brief Bind a slider with 3-stage arbitration, holdoff timing, and seq tracking.
 */
void ui_engine_bind_slider(const ui_binding_slider_opts_t *opts);

/**
 * @brief Bind a switch with programmatic update recursion guards and touch priority.
 */
void ui_engine_bind_switch(const ui_binding_switch_opts_t *opts);

/**
 * @brief Bind a push button to emit a momentary trigger command with monotonic seq_id.
 */
void ui_engine_bind_button(const ui_binding_button_opts_t *opts);

/**
 * @brief Bind a label to automatically format and display telemetry values.
 */
void ui_engine_bind_label(const ui_binding_label_opts_t *opts);

/**
 * @brief Dispatch a single telemetry message to bound widgets.
 */
void ui_engine_dispatch_telemetry(const device_telemetry_t *telem);

/**
 * @brief Build and bind the entire UI scene graph from a declarative schema.
 */
void ui_engine_build_from_schema(const widget_descriptor_t *schema, size_t count);

/**
 * @brief Retrieve a created LVGL widget by its parameter ID.
 */
lv_obj_t *ui_engine_get_widget_by_param(param_id_t param_id);

/**
 * @brief Poll bridge for all available telemetry messages and dispatch to UI.
 */
void ui_engine_pump_telemetry(void);

#ifdef __cplusplus
}
#endif

#endif // UI_BINDING_H
