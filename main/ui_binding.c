#include "ui_binding.h"
#include <stdio.h>
#include <string.h>

#define MAX_BINDINGS 8
#define MAX_LABEL_BINDINGS 16
#define MAX_PARAM_LOOKUPS 32

typedef struct {
    ui_binding_slider_opts_t opts;
    ui_arb_state_t state;
    uint32_t current_seq;
    uint32_t last_acked_seq;
    int32_t pending_val;
    lv_timer_t *holdoff_timer;
} bound_slider_t;

typedef struct {
    ui_binding_switch_opts_t opts;
    uint32_t current_seq;
    bool is_programmatic_update;
} bound_switch_t;

typedef struct {
    ui_binding_button_opts_t opts;
    uint32_t current_seq;
} bound_button_t;

typedef struct {
    ui_binding_label_opts_t opts;
} bound_label_t;

// Tier 1 Widgets
typedef struct {
    param_id_t param_id;
    lv_obj_t *arc;
    lv_obj_t *label;
    ui_arb_state_t state;
    uint32_t current_seq;
    uint32_t last_acked_seq;
    int32_t pending_val;
    lv_timer_t *holdoff_timer;
    const char *format_setting;
    const char *format_actual;
} bound_arc_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *bar;
    lv_obj_t *label;
    const char *format;
} bound_bar_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *checkbox;
    uint32_t current_seq;
    bool is_programmatic_update;
} bound_checkbox_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *led;
} bound_led_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *spinner;
} bound_spinner_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *dropdown;
    uint32_t current_seq;
} bound_dropdown_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *roller;
    uint32_t current_seq;
} bound_roller_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *spinbox;
    uint32_t current_seq;
} bound_spinbox_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *obj;
} widget_lookup_t;

static bound_slider_t s_sliders[MAX_BINDINGS];
static size_t s_num_sliders = 0;

static bound_switch_t s_switches[MAX_BINDINGS];
static size_t s_num_switches = 0;

static bound_button_t s_buttons[MAX_BINDINGS];
static size_t s_num_buttons = 0;

static bound_label_t s_labels[MAX_LABEL_BINDINGS];
static size_t s_num_labels = 0;

static bound_arc_t s_arcs[MAX_BINDINGS];
static size_t s_num_arcs = 0;

static bound_bar_t s_bars[MAX_LABEL_BINDINGS];
static size_t s_num_bars = 0;

static bound_checkbox_t s_checkboxes[MAX_BINDINGS];
static size_t s_num_checkboxes = 0;

static bound_led_t s_leds[MAX_BINDINGS];
static size_t s_num_leds = 0;

static bound_spinner_t s_spinners[MAX_BINDINGS];
static size_t s_num_spinners = 0;

static bound_dropdown_t s_dropdowns[MAX_BINDINGS];
static size_t s_num_dropdowns = 0;

static bound_roller_t s_rollers[MAX_BINDINGS];
static size_t s_num_rollers = 0;

static bound_spinbox_t s_spinboxes[MAX_BINDINGS];
static size_t s_num_spinboxes = 0;

static widget_lookup_t s_lookups[MAX_PARAM_LOOKUPS];
static size_t s_num_lookups = 0;

static uint32_t s_global_seq = 0;

static void register_lookup(param_id_t param_id, lv_obj_t *obj) {
    if (s_num_lookups < MAX_PARAM_LOOKUPS) {
        s_lookups[s_num_lookups].param_id = param_id;
        s_lookups[s_num_lookups].obj = obj;
        s_num_lookups++;
    }
}

lv_obj_t *ui_engine_get_widget_by_param(param_id_t param_id) {
    for (size_t i = 0; i < s_num_lookups; i++) {
        if (s_lookups[i].param_id == param_id) {
            return s_lookups[i].obj;
        }
    }
    return NULL;
}

// ----------------------------------------------------------------------
// Slider Arbitration Callbacks
// ----------------------------------------------------------------------
static void on_slider_holdoff_expired(lv_timer_t *timer) {
    bound_slider_t *b = (bound_slider_t *)lv_timer_get_user_data(timer);
    if (!b) return;
    if (b->last_acked_seq >= b->current_seq) {
        b->state = ARB_TRACKING;
    }
    lv_timer_pause(timer);
}

static void on_slider_event_pressed(lv_event_t *e) {
    bound_slider_t *b = (bound_slider_t *)lv_event_get_user_data(e);
    if (!b) return;
    b->state = ARB_DRAGGING;
    b->current_seq = ++s_global_seq;
    if (b->holdoff_timer) lv_timer_pause(b->holdoff_timer);
}

static void on_slider_event_value_changed(lv_event_t *e) {
    bound_slider_t *b = (bound_slider_t *)lv_event_get_user_data(e);
    if (!b || b->state != ARB_DRAGGING) return;
    b->pending_val = lv_slider_get_value(b->opts.slider);
    if (b->opts.label && b->opts.format_setting) {
        lv_label_set_text_fmt(b->opts.label, b->opts.format_setting, (long)b->pending_val);
    }
}

static void on_slider_event_released(lv_event_t *e) {
    bound_slider_t *b = (bound_slider_t *)lv_event_get_user_data(e);
    if (!b || b->state != ARB_DRAGGING) return;

    ui_command_t cmd = {
        .param_id = b->opts.param_id,
        .seq_id = b->current_seq,
        .value.i32_val = b->pending_val,
    };
    bridge_send_command_from_ui(&cmd);

    b->state = ARB_HOLDOFF;
    if (b->holdoff_timer) {
        lv_timer_reset(b->holdoff_timer);
        lv_timer_resume(b->holdoff_timer);
    }
}

// ----------------------------------------------------------------------
// Arc (Knob) Arbitration Callbacks
// ----------------------------------------------------------------------
static void on_arc_holdoff_expired(lv_timer_t *timer) {
    bound_arc_t *b = (bound_arc_t *)lv_timer_get_user_data(timer);
    if (!b) return;
    if (b->last_acked_seq >= b->current_seq) {
        b->state = ARB_TRACKING;
    }
    lv_timer_pause(timer);
}

static void on_arc_event_pressed(lv_event_t *e) {
    bound_arc_t *b = (bound_arc_t *)lv_event_get_user_data(e);
    if (!b) return;
    b->state = ARB_DRAGGING;
    b->current_seq = ++s_global_seq;
    if (b->holdoff_timer) lv_timer_pause(b->holdoff_timer);
}

static void on_arc_event_value_changed(lv_event_t *e) {
    bound_arc_t *b = (bound_arc_t *)lv_event_get_user_data(e);
    if (!b || b->state != ARB_DRAGGING) return;
    b->pending_val = lv_arc_get_value(b->arc);
    if (b->label && b->format_setting) {
        lv_label_set_text_fmt(b->label, b->format_setting, (long)b->pending_val);
    }
}

static void on_arc_event_released(lv_event_t *e) {
    bound_arc_t *b = (bound_arc_t *)lv_event_get_user_data(e);
    if (!b || b->state != ARB_DRAGGING) return;

    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = b->pending_val,
    };
    bridge_send_command_from_ui(&cmd);

    b->state = ARB_HOLDOFF;
    if (b->holdoff_timer) {
        lv_timer_reset(b->holdoff_timer);
        lv_timer_resume(b->holdoff_timer);
    }
}

// ----------------------------------------------------------------------
// Switch & Checkbox Callbacks
// ----------------------------------------------------------------------
static void on_switch_event_value_changed(lv_event_t *e) {
    bound_switch_t *b = (bound_switch_t *)lv_event_get_user_data(e);
    if (!b || b->is_programmatic_update) return;

    bool is_checked = lv_obj_has_state(b->opts.sw, LV_STATE_CHECKED);
    if (b->opts.label) {
        const char *txt = is_checked ? b->opts.label_on : b->opts.label_off;
        if (txt) lv_label_set_text(b->opts.label, txt);
    }

    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->opts.param_id,
        .seq_id = b->current_seq,
        .value.bool_val = is_checked,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_checkbox_event_value_changed(lv_event_t *e) {
    bound_checkbox_t *b = (bound_checkbox_t *)lv_event_get_user_data(e);
    if (!b || b->is_programmatic_update) return;

    bool is_checked = lv_obj_has_state(b->checkbox, LV_STATE_CHECKED);
    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.bool_val = is_checked,
    };
    bridge_send_command_from_ui(&cmd);
}

// ----------------------------------------------------------------------
// Button, Dropdown, Roller, Spinbox Callbacks
// ----------------------------------------------------------------------
static void on_button_event_clicked(lv_event_t *e) {
    bound_button_t *b = (bound_button_t *)lv_event_get_user_data(e);
    if (!b) return;

    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->opts.param_id,
        .seq_id = b->current_seq,
        .value.u32_val = b->opts.command_val,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_dropdown_event_value_changed(lv_event_t *e) {
    bound_dropdown_t *b = (bound_dropdown_t *)lv_event_get_user_data(e);
    if (!b) return;

    b->current_seq = ++s_global_seq;
    uint32_t sel = lv_dropdown_get_selected(b->dropdown);
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = (int32_t)sel,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_roller_event_value_changed(lv_event_t *e) {
    bound_roller_t *b = (bound_roller_t *)lv_event_get_user_data(e);
    if (!b) return;

    b->current_seq = ++s_global_seq;
    uint32_t sel = lv_roller_get_selected(b->roller);
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = (int32_t)sel,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_spinbox_event_value_changed(lv_event_t *e) {
    bound_spinbox_t *b = (bound_spinbox_t *)lv_event_get_user_data(e);
    if (!b) return;

    b->current_seq = ++s_global_seq;
    int32_t val = lv_spinbox_get_value(b->spinbox);
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = val,
    };
    bridge_send_command_from_ui(&cmd);
}

// ----------------------------------------------------------------------
// Engine Initialization & Direct Binding APIs
// ----------------------------------------------------------------------
void ui_engine_init(void) {
    s_num_sliders = 0;
    s_num_switches = 0;
    s_num_buttons = 0;
    s_num_labels = 0;
    s_num_arcs = 0;
    s_num_bars = 0;
    s_num_checkboxes = 0;
    s_num_leds = 0;
    s_num_spinners = 0;
    s_num_dropdowns = 0;
    s_num_rollers = 0;
    s_num_spinboxes = 0;
    s_num_lookups = 0;
    s_global_seq = 0;
}

void ui_engine_bind_slider(const ui_binding_slider_opts_t *opts) {
    if (!opts || !opts->slider || s_num_sliders >= MAX_BINDINGS) return;

    bound_slider_t *b = &s_sliders[s_num_sliders++];
    b->opts = *opts;
    b->state = ARB_TRACKING;
    b->current_seq = 0;
    b->last_acked_seq = 0;
    b->pending_val = opts->initial_val;

    uint32_t timeout = opts->holdoff_ms > 0 ? opts->holdoff_ms : DEFAULT_HOLDOFF_MS;
    b->holdoff_timer = lv_timer_create(on_slider_holdoff_expired, timeout, b);
    lv_timer_pause(b->holdoff_timer);

    lv_slider_set_range(opts->slider, opts->min, opts->max);
    lv_slider_set_value(opts->slider, opts->initial_val, LV_ANIM_OFF);

    if (opts->label && opts->format_actual) {
        lv_label_set_text_fmt(opts->label, opts->format_actual, (long)opts->initial_val);
    }

    lv_obj_add_event_cb(opts->slider, on_slider_event_pressed, LV_EVENT_PRESSED, b);
    lv_obj_add_event_cb(opts->slider, on_slider_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
    lv_obj_add_event_cb(opts->slider, on_slider_event_released, LV_EVENT_RELEASED, b);
}

void ui_engine_bind_switch(const ui_binding_switch_opts_t *opts) {
    if (!opts || !opts->sw || s_num_switches >= MAX_BINDINGS) return;

    bound_switch_t *b = &s_switches[s_num_switches++];
    b->opts = *opts;
    b->current_seq = 0;
    b->is_programmatic_update = false;

    if (opts->initial_state) {
        lv_obj_add_state(opts->sw, LV_STATE_CHECKED);
    } else {
        lv_obj_remove_state(opts->sw, LV_STATE_CHECKED);
    }

    if (opts->label) {
        const char *txt = opts->initial_state ? opts->label_on : opts->label_off;
        if (txt) lv_label_set_text(opts->label, txt);
    }

    lv_obj_add_event_cb(opts->sw, on_switch_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
}

void ui_engine_bind_button(const ui_binding_button_opts_t *opts) {
    if (!opts || !opts->button || s_num_buttons >= MAX_BINDINGS) return;

    bound_button_t *b = &s_buttons[s_num_buttons++];
    b->opts = *opts;
    b->current_seq = 0;

    lv_obj_add_event_cb(opts->button, on_button_event_clicked, LV_EVENT_CLICKED, b);
}

void ui_engine_bind_label(const ui_binding_label_opts_t *opts) {
    if (!opts || !opts->label || s_num_labels >= MAX_LABEL_BINDINGS) return;

    bound_label_t *b = &s_labels[s_num_labels++];
    b->opts = *opts;
}

// ----------------------------------------------------------------------
// Telemetry Dispatcher
// ----------------------------------------------------------------------
void ui_engine_dispatch_telemetry(const device_telemetry_t *telem) {
    if (!telem) return;

    // 1. Sliders
    for (size_t i = 0; i < s_num_sliders; i++) {
        bound_slider_t *b = &s_sliders[i];
        if (b->opts.param_id == telem->param_id) {
            b->last_acked_seq = telem->ack_seq_id;
            if (b->state == ARB_DRAGGING) continue;
            if (telem->ack_seq_id < b->current_seq) continue;
            if (b->state == ARB_HOLDOFF) continue;

            lv_slider_set_value(b->opts.slider, telem->value.i32_val, LV_ANIM_OFF);
            if (b->opts.label && b->opts.format_actual) {
                lv_label_set_text_fmt(b->opts.label, b->opts.format_actual, (long)telem->value.i32_val);
            }
        }
    }

    // 2. Arcs (Knobs / Rotary dials)
    for (size_t i = 0; i < s_num_arcs; i++) {
        bound_arc_t *b = &s_arcs[i];
        if (b->param_id == telem->param_id) {
            b->last_acked_seq = telem->ack_seq_id;
            if (b->state == ARB_DRAGGING) continue;
            if (telem->ack_seq_id < b->current_seq) continue;
            if (b->state == ARB_HOLDOFF) continue;

            lv_arc_set_value(b->arc, telem->value.i32_val);
            if (b->label && b->format_actual) {
                lv_label_set_text_fmt(b->label, b->format_actual, (long)telem->value.i32_val);
            }
        }
    }

    // 3. Bars (Progress / Level indicators)
    for (size_t i = 0; i < s_num_bars; i++) {
        bound_bar_t *b = &s_bars[i];
        if (b->param_id == telem->param_id) {
            lv_bar_set_value(b->bar, telem->value.i32_val, LV_ANIM_ON);
            if (b->label && b->format) {
                lv_label_set_text_fmt(b->label, b->format, (long)telem->value.i32_val);
            }
        }
    }

    // 4. Switches
    for (size_t i = 0; i < s_num_switches; i++) {
        bound_switch_t *b = &s_switches[i];
        if (b->opts.param_id == telem->param_id) {
            if (!lv_obj_has_state(b->opts.sw, LV_STATE_PRESSED)) {
                b->is_programmatic_update = true;
                if (telem->value.bool_val) {
                    lv_obj_add_state(b->opts.sw, LV_STATE_CHECKED);
                    if (b->opts.label && b->opts.label_on) {
                        lv_label_set_text(b->opts.label, b->opts.label_on);
                    }
                } else {
                    lv_obj_remove_state(b->opts.sw, LV_STATE_CHECKED);
                    if (b->opts.label && b->opts.label_off) {
                        lv_label_set_text(b->opts.label, b->opts.label_off);
                    }
                }
                b->is_programmatic_update = false;
            }
        }
    }

    // 5. Checkboxes
    for (size_t i = 0; i < s_num_checkboxes; i++) {
        bound_checkbox_t *b = &s_checkboxes[i];
        if (b->param_id == telem->param_id) {
            if (!lv_obj_has_state(b->checkbox, LV_STATE_PRESSED)) {
                b->is_programmatic_update = true;
                if (telem->value.bool_val) {
                    lv_obj_add_state(b->checkbox, LV_STATE_CHECKED);
                } else {
                    lv_obj_remove_state(b->checkbox, LV_STATE_CHECKED);
                }
                b->is_programmatic_update = false;
            }
        }
    }

    // 6. LEDs
    for (size_t i = 0; i < s_num_leds; i++) {
        bound_led_t *b = &s_leds[i];
        if (b->param_id == telem->param_id) {
            if (telem->value.bool_val) {
                lv_led_on(b->led);
            } else {
                lv_led_off(b->led);
            }
        }
    }

    // 7. Spinners
    for (size_t i = 0; i < s_num_spinners; i++) {
        bound_spinner_t *b = &s_spinners[i];
        if (b->param_id == telem->param_id) {
            if (telem->value.bool_val) {
                lv_obj_remove_flag(b->spinner, LV_OBJ_FLAG_HIDDEN);
            } else {
                lv_obj_add_flag(b->spinner, LV_OBJ_FLAG_HIDDEN);
            }
        }
    }

    // 8. Dropdowns
    for (size_t i = 0; i < s_num_dropdowns; i++) {
        bound_dropdown_t *b = &s_dropdowns[i];
        if (b->param_id == telem->param_id) {
            lv_dropdown_set_selected(b->dropdown, (uint32_t)telem->value.i32_val);
        }
    }

    // 9. Rollers
    for (size_t i = 0; i < s_num_rollers; i++) {
        bound_roller_t *b = &s_rollers[i];
        if (b->param_id == telem->param_id) {
            lv_roller_set_selected(b->roller, (uint32_t)telem->value.i32_val, LV_ANIM_OFF);
        }
    }

    // 10. Spinboxes
    for (size_t i = 0; i < s_num_spinboxes; i++) {
        bound_spinbox_t *b = &s_spinboxes[i];
        if (b->param_id == telem->param_id) {
            lv_spinbox_set_value(b->spinbox, telem->value.i32_val);
        }
    }

    // 11. Labels
    for (size_t i = 0; i < s_num_labels; i++) {
        bound_label_t *b = &s_labels[i];
        if (b->opts.param_id == telem->param_id && b->opts.format) {
            lv_label_set_text_fmt(b->opts.label, b->opts.format, (unsigned long)telem->value.u32_val);
        }
    }
}

void ui_engine_pump_telemetry(void) {
    device_telemetry_t telem;
    while (bridge_recv_telemetry_in_ui(&telem)) {
        ui_engine_dispatch_telemetry(&telem);
    }
}

static lv_align_t map_align(ui_align_t a) {
    switch (a) {
    case UI_ALIGN_TOP_LEFT:     return LV_ALIGN_TOP_LEFT;
    case UI_ALIGN_TOP_MID:      return LV_ALIGN_TOP_MID;
    case UI_ALIGN_TOP_RIGHT:    return LV_ALIGN_TOP_RIGHT;
    case UI_ALIGN_CENTER:       return LV_ALIGN_CENTER;
    case UI_ALIGN_BOTTOM_LEFT:  return LV_ALIGN_BOTTOM_LEFT;
    case UI_ALIGN_BOTTOM_MID:   return LV_ALIGN_BOTTOM_MID;
    case UI_ALIGN_BOTTOM_RIGHT: return LV_ALIGN_BOTTOM_RIGHT;
    default:                    return LV_ALIGN_CENTER;
    }
}

// ----------------------------------------------------------------------
// Declarative Schema Builder (builds all Tier 1 widgets from Rust schema)
// ----------------------------------------------------------------------
void ui_engine_build_from_schema(const widget_descriptor_t *schema, size_t count) {
    if (!schema || count == 0) return;
    s_num_lookups = 0;
    lv_obj_t *scr = lv_screen_active();

    for (size_t i = 0; i < count; i++) {
        const widget_descriptor_t *d = &schema[i];
        lv_align_t align = map_align(d->align);

        switch (d->widget_type) {
        case WIDGET_TYPE_SWITCH: {
            lv_obj_t *sw = lv_switch_create(scr);
            lv_obj_align(sw, align, d->offset_x, d->offset_y);

            lv_obj_t *lbl = lv_label_create(scr);
            lv_label_set_text(lbl, d->label_secondary ? d->label_secondary : "OFF");
            lv_obj_align_to(lbl, sw, LV_ALIGN_OUT_RIGHT_MID, 15, 0);

            ui_engine_bind_switch(&(ui_binding_switch_opts_t){
                .sw = sw,
                .label = lbl,
                .param_id = d->param_id,
                .initial_state = (d->initial_val != 0),
                .label_on = d->label_primary,
                .label_off = d->label_secondary,
            });
            register_lookup(d->param_id, sw);
            break;
        }

        case WIDGET_TYPE_SLIDER: {
            lv_obj_t *lbl = lv_label_create(scr);
            lv_obj_align(lbl, align, d->offset_x, d->offset_y - 40);

            lv_obj_t *slider = lv_slider_create(scr);
            lv_obj_align(slider, align, d->offset_x, d->offset_y);

            ui_engine_bind_slider(&(ui_binding_slider_opts_t){
                .slider = slider,
                .label = lbl,
                .param_id = d->param_id,
                .min = d->min_val,
                .max = d->max_val,
                .initial_val = d->initial_val,
                .format_setting = d->label_primary,
                .format_actual = d->label_secondary,
                .holdoff_ms = d->holdoff_ms,
            });
            register_lookup(d->param_id, slider);
            break;
        }

        case WIDGET_TYPE_BUTTON: {
            lv_obj_t *btn = lv_button_create(scr);
            lv_obj_align(btn, align, d->offset_x, d->offset_y);

            lv_obj_t *btn_lbl = lv_label_create(btn);
            if (d->label_primary) {
                lv_label_set_text(btn_lbl, d->label_primary);
            }

            ui_engine_bind_button(&(ui_binding_button_opts_t){
                .button = btn,
                .param_id = d->param_id,
                .command_val = (uint32_t)d->initial_val,
            });
            register_lookup(d->param_id, btn);
            break;
        }

        case WIDGET_TYPE_LABEL: {
            lv_obj_t *lbl = lv_label_create(scr);
            lv_obj_align(lbl, align, d->offset_x, d->offset_y);
            if (d->label_primary) {
                lv_label_set_text_fmt(lbl, d->label_primary, (unsigned long)d->initial_val);
            }

            ui_engine_bind_label(&(ui_binding_label_opts_t){
                .label = lbl,
                .param_id = d->param_id,
                .format = d->label_primary,
            });
            register_lookup(d->param_id, lbl);
            break;
        }

        case WIDGET_TYPE_ARC: {
            lv_obj_t *arc = lv_arc_create(scr);
            lv_obj_align(arc, align, d->offset_x, d->offset_y);
            lv_arc_set_range(arc, d->min_val, d->max_val);
            lv_arc_set_value(arc, d->initial_val);

            lv_obj_t *lbl = NULL;
            if (d->label_secondary) {
                lbl = lv_label_create(scr);
                lv_obj_align_to(lbl, arc, LV_ALIGN_CENTER, 0, 0);
                lv_label_set_text_fmt(lbl, d->label_secondary, (long)d->initial_val);
            }

            if (s_num_arcs < MAX_BINDINGS) {
                bound_arc_t *b = &s_arcs[s_num_arcs++];
                b->param_id = d->param_id;
                b->arc = arc;
                b->label = lbl;
                b->state = ARB_TRACKING;
                b->current_seq = 0;
                b->last_acked_seq = 0;
                b->pending_val = d->initial_val;
                b->format_setting = d->label_primary;
                b->format_actual = d->label_secondary;

                uint32_t timeout = d->holdoff_ms > 0 ? d->holdoff_ms : DEFAULT_HOLDOFF_MS;
                b->holdoff_timer = lv_timer_create(on_arc_holdoff_expired, timeout, b);
                lv_timer_pause(b->holdoff_timer);

                lv_obj_add_event_cb(arc, on_arc_event_pressed, LV_EVENT_PRESSED, b);
                lv_obj_add_event_cb(arc, on_arc_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
                lv_obj_add_event_cb(arc, on_arc_event_released, LV_EVENT_RELEASED, b);
            }
            register_lookup(d->param_id, arc);
            break;
        }

        case WIDGET_TYPE_BAR: {
            lv_obj_t *bar = lv_bar_create(scr);
            lv_obj_align(bar, align, d->offset_x, d->offset_y);
            lv_bar_set_range(bar, d->min_val, d->max_val);
            lv_bar_set_value(bar, d->initial_val, LV_ANIM_OFF);

            lv_obj_t *lbl = NULL;
            if (d->label_primary) {
                lbl = lv_label_create(scr);
                lv_obj_align_to(lbl, bar, LV_ALIGN_OUT_TOP_MID, 0, -10);
                lv_label_set_text_fmt(lbl, d->label_primary, (long)d->initial_val);
            }

            if (s_num_bars < MAX_LABEL_BINDINGS) {
                bound_bar_t *b = &s_bars[s_num_bars++];
                b->param_id = d->param_id;
                b->bar = bar;
                b->label = lbl;
                b->format = d->label_primary;
            }
            register_lookup(d->param_id, bar);
            break;
        }

        case WIDGET_TYPE_CHECKBOX: {
            lv_obj_t *cb = lv_checkbox_create(scr);
            lv_obj_align(cb, align, d->offset_x, d->offset_y);
            if (d->label_primary) {
                lv_checkbox_set_text(cb, d->label_primary);
            }
            if (d->initial_val) {
                lv_obj_add_state(cb, LV_STATE_CHECKED);
            }

            if (s_num_checkboxes < MAX_BINDINGS) {
                bound_checkbox_t *b = &s_checkboxes[s_num_checkboxes++];
                b->param_id = d->param_id;
                b->checkbox = cb;
                b->current_seq = 0;
                b->is_programmatic_update = false;

                lv_obj_add_event_cb(cb, on_checkbox_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, cb);
            break;
        }

        case WIDGET_TYPE_LED: {
            lv_obj_t *led = lv_led_create(scr);
            lv_obj_align(led, align, d->offset_x, d->offset_y);
            if (d->initial_val) {
                lv_led_on(led);
            } else {
                lv_led_off(led);
            }

            if (s_num_leds < MAX_BINDINGS) {
                bound_led_t *b = &s_leds[s_num_leds++];
                b->param_id = d->param_id;
                b->led = led;
            }
            register_lookup(d->param_id, led);
            break;
        }

        case WIDGET_TYPE_SPINNER: {
            lv_obj_t *sp = lv_spinner_create(scr);
            lv_obj_set_size(sp, 40, 40);
            lv_obj_align(sp, align, d->offset_x, d->offset_y);
            if (!d->initial_val) {
                lv_obj_add_flag(sp, LV_OBJ_FLAG_HIDDEN);
            }

            if (s_num_spinners < MAX_BINDINGS) {
                bound_spinner_t *b = &s_spinners[s_num_spinners++];
                b->param_id = d->param_id;
                b->spinner = sp;
            }
            register_lookup(d->param_id, sp);
            break;
        }

        case WIDGET_TYPE_DROPDOWN: {
            lv_obj_t *dd = lv_dropdown_create(scr);
            lv_obj_align(dd, align, d->offset_x, d->offset_y);
            if (d->label_primary) {
                lv_dropdown_set_options(dd, d->label_primary);
            }
            lv_dropdown_set_selected(dd, (uint32_t)d->initial_val);

            if (s_num_dropdowns < MAX_BINDINGS) {
                bound_dropdown_t *b = &s_dropdowns[s_num_dropdowns++];
                b->param_id = d->param_id;
                b->dropdown = dd;
                b->current_seq = 0;

                lv_obj_add_event_cb(dd, on_dropdown_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, dd);
            break;
        }

        case WIDGET_TYPE_ROLLER: {
            lv_obj_t *roller = lv_roller_create(scr);
            lv_obj_align(roller, align, d->offset_x, d->offset_y);
            if (d->label_primary) {
                lv_roller_set_options(roller, d->label_primary, LV_ROLLER_MODE_NORMAL);
            }
            lv_roller_set_selected(roller, (uint32_t)d->initial_val, LV_ANIM_OFF);

            if (s_num_rollers < MAX_BINDINGS) {
                bound_roller_t *b = &s_rollers[s_num_rollers++];
                b->param_id = d->param_id;
                b->roller = roller;
                b->current_seq = 0;

                lv_obj_add_event_cb(roller, on_roller_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, roller);
            break;
        }

        case WIDGET_TYPE_SPINBOX: {
            lv_obj_t *sb = lv_spinbox_create(scr);
            lv_obj_align(sb, align, d->offset_x, d->offset_y);
            lv_spinbox_set_range(sb, d->min_val, d->max_val);
            lv_spinbox_set_digit_format(sb, 4, 0);
            lv_spinbox_set_step(sb, 1);
            lv_spinbox_set_value(sb, d->initial_val);

            if (s_num_spinboxes < MAX_BINDINGS) {
                bound_spinbox_t *b = &s_spinboxes[s_num_spinboxes++];
                b->param_id = d->param_id;
                b->spinbox = sb;
                b->current_seq = 0;

                lv_obj_add_event_cb(sb, on_spinbox_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, sb);
            break;
        }

        default:
            break;
        }
    }
}
