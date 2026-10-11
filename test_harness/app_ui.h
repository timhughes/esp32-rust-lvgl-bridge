#ifndef APP_UI_H
#define APP_UI_H

#include "lvgl.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    lv_obj_t *slider;
    lv_obj_t *slider_label;
    lv_obj_t *power_switch;
    lv_obj_t *switch_label;
    lv_obj_t *reset_btn;
    lv_obj_t *counter_label;
} ui_context_t;

void setup_test_ui(void);
void pump_incoming_telemetry(void);
ui_context_t *get_test_ui(void);

#ifdef __cplusplus
}
#endif

#endif // APP_UI_H
