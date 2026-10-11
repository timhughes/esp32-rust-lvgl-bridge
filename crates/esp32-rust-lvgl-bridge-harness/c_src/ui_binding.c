#include "ui_binding.h"
#include <stdio.h>
#include <string.h>

static inline void ui_set_obj_hidden(lv_obj_t *obj, bool hidden) {
    if (!obj) return;
    if (hidden) {
        lv_obj_add_flag(obj, LV_OBJ_FLAG_HIDDEN);
    } else {
        lv_obj_remove_flag(obj, LV_OBJ_FLAG_HIDDEN);
    }
}

#if LV_USE_FLEX
static void apply_layout_flow(lv_obj_t *obj, layout_flow_t flow_mode) {
    if (!obj || flow_mode == LAYOUT_FLOW_NONE) return;
    lv_flex_flow_t flow = LV_FLEX_FLOW_ROW;
    switch (flow_mode) {
    case LAYOUT_FLOW_ROW:            flow = LV_FLEX_FLOW_ROW; break;
    case LAYOUT_FLOW_COLUMN:         flow = LV_FLEX_FLOW_COLUMN; break;
    case LAYOUT_FLOW_ROW_WRAP:       flow = LV_FLEX_FLOW_ROW_WRAP; break;
    case LAYOUT_FLOW_ROW_REVERSE:    flow = LV_FLEX_FLOW_ROW_REVERSE; break;
    case LAYOUT_FLOW_COLUMN_WRAP:    flow = LV_FLEX_FLOW_COLUMN_WRAP; break;
    case LAYOUT_FLOW_COLUMN_REVERSE: flow = LV_FLEX_FLOW_COLUMN_REVERSE; break;
    default: return;
    }
    lv_obj_set_flex_flow(obj, flow);
}
#endif

#define MAX_BINDINGS 16
#define MAX_LABEL_BINDINGS 16
#define MAX_PARAM_LOOKUPS 64

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
    bool is_programmatic_update;
} bound_dropdown_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *roller;
    uint32_t current_seq;
    bool is_programmatic_update;
} bound_roller_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *spinbox;
    uint32_t current_seq;
    bool is_programmatic_update;
} bound_spinbox_t;

// Tier 2 Widgets
typedef struct {
    param_id_t param_id;
    lv_obj_t *textarea;
    uint32_t current_seq;
} bound_textarea_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *keyboard;
    uint32_t current_seq;
} bound_keyboard_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *btnmatrix;
    uint32_t current_seq;
} bound_btnmatrix_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *msgbox;
    uint32_t current_seq;
} bound_msgbox_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *scale;
    lv_obj_t *needle;
} bound_scale_t;

// Tier 3 Widgets
typedef struct {
    param_id_t param_id;
    lv_obj_t *tabview;
    lv_obj_t *tabs[8];
    size_t num_tabs;
    uint32_t current_seq;
    bool is_programmatic_update;
} bound_tabview_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *tileview;
    lv_obj_t *tiles[8];
    size_t num_tiles;
} bound_tileview_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *menu;
} bound_menu_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *list;
} bound_list_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *chart;
    lv_chart_series_t *series;
} bound_chart_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *table;
} bound_table_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *calendar;
    uint32_t current_seq;
} bound_calendar_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *canvas;
    uint8_t *cbuf;
} bound_canvas_t;

// Phase 1 Widgets
typedef struct {
    param_id_t param_id;
    lv_obj_t *image;
} bound_image_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *line;
    lv_point_precise_t points[16];
    uint32_t point_cnt;
} bound_line_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *arclabel;
} bound_arclabel_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *qrcode;
} bound_qrcode_t;

// Phase 2 Widgets
typedef struct {
    param_id_t param_id;
    lv_obj_t *win;
    lv_obj_t *content;
} bound_win_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *imgbtn;
    uint32_t current_seq;
    uint32_t command_val;
} bound_imagebutton_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *spangroup;
} bound_spangroup_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *animimg;
} bound_animimage_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *barcode;
} bound_barcode_t;

typedef struct {
    param_id_t param_id;
    lv_obj_t *obj;
} widget_lookup_t;

// Static instances
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

// Tier 2
static bound_textarea_t s_textareas[MAX_BINDINGS];
static size_t s_num_textareas = 0;

static bound_keyboard_t s_keyboards[MAX_BINDINGS];
static size_t s_num_keyboards = 0;

static bound_btnmatrix_t s_btnmatrices[MAX_BINDINGS];
static size_t s_num_btnmatrices = 0;

static bound_msgbox_t s_msgboxes[MAX_BINDINGS];
static size_t s_num_msgboxes = 0;

static bound_scale_t s_scales[MAX_BINDINGS];
static size_t s_num_scales = 0;

// Tier 3
static bound_tabview_t s_tabviews[MAX_BINDINGS];
static size_t s_num_tabviews = 0;

static bound_tileview_t s_tileviews[MAX_BINDINGS];
static size_t s_num_tileviews = 0;

static bound_menu_t s_menus[MAX_BINDINGS];
static size_t s_num_menus = 0;

static bound_list_t s_lists[MAX_BINDINGS];
static size_t s_num_lists = 0;

static bound_chart_t s_charts[MAX_BINDINGS];
static size_t s_num_charts = 0;

static bound_table_t s_tables[MAX_BINDINGS];
static size_t s_num_tables = 0;

static bound_calendar_t s_calendars[MAX_BINDINGS];
static size_t s_num_calendars = 0;

static bound_canvas_t s_canvases[MAX_BINDINGS];
static size_t s_num_canvases = 0;

// Phase 1
static bound_image_t s_images[MAX_BINDINGS];
static size_t s_num_images = 0;

static bound_line_t s_lines[MAX_BINDINGS];
static size_t s_num_lines = 0;

static bound_arclabel_t s_arclabels[MAX_BINDINGS];
static size_t s_num_arclabels = 0;

static bound_qrcode_t s_qrcodes[MAX_BINDINGS];
static size_t s_num_qrcodes = 0;

// Phase 2
static bound_win_t s_wins[MAX_BINDINGS];
static size_t s_num_wins = 0;

static bound_imagebutton_t s_imgbtns[MAX_BINDINGS];
static size_t s_num_imgbtns = 0;

static bound_spangroup_t s_spangroups[MAX_BINDINGS];
static size_t s_num_spangroups = 0;

static bound_animimage_t s_animimages[MAX_BINDINGS];
static size_t s_num_animimages = 0;

static bound_barcode_t s_barcodes[MAX_BINDINGS];
static size_t s_num_barcodes = 0;

// Button matrix map storage
static char s_btnmatrix_str_bufs[MAX_BINDINGS][256];
static const char *s_btnmatrix_maps[MAX_BINDINGS][32];

// Canvas static draw buffer (100x100 RGB565)
static uint8_t s_canvas_buffers[2][100 * 100 * 2];

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
    if (!b || b->is_programmatic_update) return;

    uint32_t sel = lv_dropdown_get_selected(b->dropdown);
    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = (int32_t)sel,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_roller_event_value_changed(lv_event_t *e) {
    bound_roller_t *b = (bound_roller_t *)lv_event_get_user_data(e);
    if (!b || b->is_programmatic_update) return;

    uint32_t sel = lv_roller_get_selected(b->roller);
    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = (int32_t)sel,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_spinbox_event_value_changed(lv_event_t *e) {
    bound_spinbox_t *b = (bound_spinbox_t *)lv_event_get_user_data(e);
    if (!b || b->is_programmatic_update) return;

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
// Tier 2 Callbacks: TextArea, Keyboard, ButtonMatrix, MsgBox
// ----------------------------------------------------------------------
static void on_textarea_event_value_changed(lv_event_t *e) {
    bound_textarea_t *b = (bound_textarea_t *)lv_event_get_user_data(e);
    if (!b) return;

    const char *txt = lv_textarea_get_text(b->textarea);
    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
    };
    if (txt) {
        strncpy(cmd.value.str_val, txt, sizeof(cmd.value.str_val) - 1);
        cmd.value.str_val[sizeof(cmd.value.str_val) - 1] = '\0';
    }
    bridge_send_command_from_ui(&cmd);
}

static void on_textarea_focused(lv_event_t *e) {
    lv_obj_t *ta = lv_event_get_current_target(e);
    for (size_t i = 0; i < s_num_keyboards; i++) {
        if (s_keyboards[i].keyboard) {
            lv_keyboard_set_textarea(s_keyboards[i].keyboard, ta);
        }
    }
}

static void on_keyboard_event(lv_event_t *e) {
    bound_keyboard_t *b = (bound_keyboard_t *)lv_event_get_user_data(e);
    if (!b) return;

    lv_event_code_t code = lv_event_get_code(e);
    if (code == LV_EVENT_READY || code == LV_EVENT_CANCEL) {
        b->current_seq = ++s_global_seq;
        ui_command_t cmd = {
            .param_id = b->param_id,
            .seq_id = b->current_seq,
            .value.i32_val = (code == LV_EVENT_READY) ? 1 : 2,
        };
        bridge_send_command_from_ui(&cmd);
    }
}

static void on_btnmatrix_event_value_changed(lv_event_t *e) {
    bound_btnmatrix_t *b = (bound_btnmatrix_t *)lv_event_get_user_data(e);
    if (!b) return;

    uint32_t btn_id = lv_buttonmatrix_get_selected_button(b->btnmatrix);
    const char *txt = lv_buttonmatrix_get_button_text(b->btnmatrix, btn_id);
    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = (int32_t)btn_id,
    };
    if (txt) {
        strncpy(cmd.value.str_val, txt, sizeof(cmd.value.str_val) - 1);
        cmd.value.str_val[sizeof(cmd.value.str_val) - 1] = '\0';
    }
    bridge_send_command_from_ui(&cmd);
}

static void on_msgbox_close_clicked(lv_event_t *e) {
    bound_msgbox_t *b = (bound_msgbox_t *)lv_event_get_user_data(e);
    if (!b) return;

    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = 0,
    };
    bridge_send_command_from_ui(&cmd);
    ui_set_obj_hidden(b->msgbox, true);
}

static void on_msgbox_ok_clicked(lv_event_t *e) {
    bound_msgbox_t *b = (bound_msgbox_t *)lv_event_get_user_data(e);
    if (!b) return;

    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = 1,
    };
    bridge_send_command_from_ui(&cmd);
    ui_set_obj_hidden(b->msgbox, true);
}

static void on_win_close_clicked(lv_event_t *e) {
    bound_win_t *b = (bound_win_t *)lv_event_get_user_data(e);
    if (!b) return;

    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = ++s_global_seq,
        .value.bool_val = false,
    };
    bridge_send_command_from_ui(&cmd);
    ui_set_obj_hidden(b->win, true);
}

static void on_imgbtn_event_clicked(lv_event_t *e) {
    bound_imagebutton_t *b = (bound_imagebutton_t *)lv_event_get_user_data(e);
    if (!b) return;

    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = ++b->current_seq,
        .value = { .u32_val = b->command_val ? b->command_val : 1 },
    };
    bridge_send_command_from_ui(&cmd);
}

// ----------------------------------------------------------------------
// Tier 3 Callbacks: TabView, Calendar, List
// ----------------------------------------------------------------------
static void on_tabview_event_value_changed(lv_event_t *e) {
    bound_tabview_t *b = (bound_tabview_t *)lv_event_get_user_data(e);
    if (!b || b->is_programmatic_update) return;

    uint32_t active = lv_tabview_get_tab_active(b->tabview);
    b->current_seq = ++s_global_seq;
    ui_command_t cmd = {
        .param_id = b->param_id,
        .seq_id = b->current_seq,
        .value.i32_val = (int32_t)active,
    };
    bridge_send_command_from_ui(&cmd);
}

static void on_calendar_event_value_changed(lv_event_t *e) {
    bound_calendar_t *b = (bound_calendar_t *)lv_event_get_user_data(e);
    if (!b) return;

    lv_calendar_date_t date;
    if (lv_calendar_get_pressed_date(b->calendar, &date) == LV_RESULT_OK) {
        uint32_t enc = (uint32_t)date.year * 10000 + (uint32_t)date.month * 100 + (uint32_t)date.day;
        b->current_seq = ++s_global_seq;
        ui_command_t cmd = {
            .param_id = b->param_id,
            .seq_id = b->current_seq,
            .value.u32_val = enc,
        };
        bridge_send_command_from_ui(&cmd);
    }
}

static void on_list_button_clicked(lv_event_t *e) {
    param_id_t param_id = (param_id_t)(intptr_t)lv_event_get_user_data(e);
    lv_obj_t *target = lv_event_get_current_target(e);
    const char *txt = lv_list_get_button_text(lv_obj_get_parent(target), target);

    ui_command_t cmd = {
        .param_id = param_id,
        .seq_id = ++s_global_seq,
        .value.i32_val = 1,
    };
    if (txt) {
        strncpy(cmd.value.str_val, txt, sizeof(cmd.value.str_val) - 1);
        cmd.value.str_val[sizeof(cmd.value.str_val) - 1] = '\0';
    }
    bridge_send_command_from_ui(&cmd);
}

// ----------------------------------------------------------------------
// Helper to parse button matrix maps
// ----------------------------------------------------------------------
static const char **parse_btnmatrix_map(size_t idx, const char *raw) {
    if (!raw || idx >= MAX_BINDINGS) return NULL;
    strncpy(s_btnmatrix_str_bufs[idx], raw, sizeof(s_btnmatrix_str_bufs[idx]) - 1);
    s_btnmatrix_str_bufs[idx][sizeof(s_btnmatrix_str_bufs[idx]) - 1] = '\0';

    size_t map_idx = 0;
    char *p = s_btnmatrix_str_bufs[idx];
    while (*p && map_idx < 30) {
        while (*p == ' ') p++;
        if (!*p) break;
        if (*p == '\\' && *(p + 1) == 'n') {
            s_btnmatrix_maps[idx][map_idx++] = "\n";
            p += 2;
            continue;
        }
        if (*p == '\n') {
            s_btnmatrix_maps[idx][map_idx++] = "\n";
            p++;
            continue;
        }
        char *start = p;
        while (*p && *p != ' ' && *p != '\n' && !(*p == '\\' && *(p + 1) == 'n')) p++;
        char save = *p;
        *p = '\0';
        s_btnmatrix_maps[idx][map_idx++] = start;
        if (save != '\0') p++;
    }
    s_btnmatrix_maps[idx][map_idx] = "";
    return s_btnmatrix_maps[idx];
}

static uint32_t parse_line_points(const char *str, lv_point_precise_t *points, uint32_t max_points) {
    if (!str || !points || max_points == 0) return 0;
    uint32_t count = 0;
    const char *p = str;
    while (*p && count < max_points) {
        while (*p == ' ' || *p == ';' || *p == '\n') p++;
        if (!*p) break;
        int x = 0, y = 0;
        if (sscanf(p, "%d,%d", &x, &y) == 2) {
            points[count].x = (lv_value_precise_t)x;
            points[count].y = (lv_value_precise_t)y;
            count++;
        }
        while (*p && *p != ';' && *p != '\n') p++;
        if (*p) p++;
    }
    return count;
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
    // Tier 2
    s_num_textareas = 0;
    s_num_keyboards = 0;
    s_num_btnmatrices = 0;
    s_num_msgboxes = 0;
    s_num_scales = 0;
    // Tier 3
    s_num_tabviews = 0;
    s_num_tileviews = 0;
    s_num_menus = 0;
    s_num_lists = 0;
    s_num_charts = 0;
    s_num_tables = 0;
    s_num_calendars = 0;
    s_num_canvases = 0;
    // Phase 1
    s_num_images = 0;
    s_num_lines = 0;
    s_num_arclabels = 0;
    s_num_qrcodes = 0;
    // Phase 2
    s_num_wins = 0;
    s_num_imgbtns = 0;
    s_num_spangroups = 0;
    s_num_animimages = 0;
    s_num_barcodes = 0;

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
// Telemetry Dispatcher (DeviceTelemetry -> UI widgets)
// ----------------------------------------------------------------------
void ui_engine_dispatch_telemetry(const device_telemetry_t *telem) {
    if (!telem) return;

    // 1. Sliders (3-stage arbitration)
    for (size_t i = 0; i < s_num_sliders; i++) {
        bound_slider_t *b = &s_sliders[i];
        if (b->opts.param_id == telem->param_id) {
            b->last_acked_seq = telem->ack_seq_id;
            if (b->state == ARB_TRACKING) {
                lv_slider_set_value(b->opts.slider, telem->value.i32_val, LV_ANIM_OFF);
                if (b->opts.label && b->opts.format_actual) {
                    lv_label_set_text_fmt(b->opts.label, b->opts.format_actual, (long)telem->value.i32_val);
                }
            } else if (b->state == ARB_HOLDOFF) {
                if (b->last_acked_seq >= b->current_seq) {
                    b->state = ARB_TRACKING;
                    lv_slider_set_value(b->opts.slider, telem->value.i32_val, LV_ANIM_OFF);
                    if (b->opts.label && b->opts.format_actual) {
                        lv_label_set_text_fmt(b->opts.label, b->opts.format_actual, (long)telem->value.i32_val);
                    }
                }
            }
        }
    }

    // 2. Arcs (Knob arbitration)
    for (size_t i = 0; i < s_num_arcs; i++) {
        bound_arc_t *b = &s_arcs[i];
        if (b->param_id == telem->param_id) {
            b->last_acked_seq = telem->ack_seq_id;
            if (b->state == ARB_TRACKING) {
                lv_arc_set_value(b->arc, telem->value.i32_val);
                if (b->label && b->format_actual) {
                    lv_label_set_text_fmt(b->label, b->format_actual, (long)telem->value.i32_val);
                }
            } else if (b->state == ARB_HOLDOFF) {
                if (b->last_acked_seq >= b->current_seq) {
                    b->state = ARB_TRACKING;
                    lv_arc_set_value(b->arc, telem->value.i32_val);
                    if (b->label && b->format_actual) {
                        lv_label_set_text_fmt(b->label, b->format_actual, (long)telem->value.i32_val);
                    }
                }
            }
        }
    }

    // 3. Switches
    for (size_t i = 0; i < s_num_switches; i++) {
        bound_switch_t *b = &s_switches[i];
        if (b->opts.param_id == telem->param_id) {
            if (!lv_obj_has_state(b->opts.sw, LV_STATE_PRESSED)) {
                b->is_programmatic_update = true;
                if (telem->value.bool_val) {
                    lv_obj_add_state(b->opts.sw, LV_STATE_CHECKED);
                } else {
                    lv_obj_remove_state(b->opts.sw, LV_STATE_CHECKED);
                }
                if (b->opts.label) {
                    const char *txt = telem->value.bool_val ? b->opts.label_on : b->opts.label_off;
                    if (txt) lv_label_set_text(b->opts.label, txt);
                }
                b->is_programmatic_update = false;
            }
        }
    }

    // 4. Bars
    for (size_t i = 0; i < s_num_bars; i++) {
        bound_bar_t *b = &s_bars[i];
        if (b->param_id == telem->param_id) {
            lv_bar_set_value(b->bar, telem->value.i32_val, LV_ANIM_OFF);
            if (b->label && b->format) {
                lv_label_set_text_fmt(b->label, b->format, (long)telem->value.i32_val);
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
            ui_set_obj_hidden(b->spinner, !telem->value.bool_val);
        }
    }

    // 8. Dropdowns
    for (size_t i = 0; i < s_num_dropdowns; i++) {
        bound_dropdown_t *b = &s_dropdowns[i];
        if (b->param_id == telem->param_id) {
            uint32_t target_idx = (uint32_t)telem->value.i32_val;
            if (lv_dropdown_get_selected(b->dropdown) != target_idx) {
                b->is_programmatic_update = true;
                lv_dropdown_set_selected(b->dropdown, target_idx);
                b->is_programmatic_update = false;
            }
        }
    }

    // 9. Rollers
    for (size_t i = 0; i < s_num_rollers; i++) {
        bound_roller_t *b = &s_rollers[i];
        if (b->param_id == telem->param_id) {
            uint32_t target_idx = (uint32_t)telem->value.i32_val;
            if (lv_roller_get_selected(b->roller) != target_idx) {
                b->is_programmatic_update = true;
                lv_roller_set_selected(b->roller, target_idx, LV_ANIM_OFF);
                b->is_programmatic_update = false;
            }
        }
    }

    // 10. Spinboxes
    for (size_t i = 0; i < s_num_spinboxes; i++) {
        bound_spinbox_t *b = &s_spinboxes[i];
        if (b->param_id == telem->param_id) {
            if (lv_spinbox_get_value(b->spinbox) != telem->value.i32_val) {
                b->is_programmatic_update = true;
                lv_spinbox_set_value(b->spinbox, telem->value.i32_val);
                b->is_programmatic_update = false;
            }
        }
    }

    // 11. Labels
    for (size_t i = 0; i < s_num_labels; i++) {
        bound_label_t *b = &s_labels[i];
        if (b->opts.param_id == telem->param_id && b->opts.format) {
            lv_label_set_text_fmt(b->opts.label, b->opts.format, (unsigned long)telem->value.u32_val);
        }
    }

    // 12. TextAreas (Tier 2)
    for (size_t i = 0; i < s_num_textareas; i++) {
        bound_textarea_t *b = &s_textareas[i];
        if (b->param_id == telem->param_id) {
            lv_textarea_set_text(b->textarea, telem->value.str_val);
        }
    }

    // 13. MsgBoxes (Tier 2)
    for (size_t i = 0; i < s_num_msgboxes; i++) {
        bound_msgbox_t *b = &s_msgboxes[i];
        if (b->param_id == telem->param_id) {
            ui_set_obj_hidden(b->msgbox, !telem->value.bool_val);
        }
    }

    // 14. Scales (Tier 2)
    for (size_t i = 0; i < s_num_scales; i++) {
        bound_scale_t *b = &s_scales[i];
        if (b->param_id == telem->param_id && b->needle) {
            lv_scale_set_line_needle_value(b->scale, b->needle, -25, telem->value.i32_val);
        }
    }

    // 15. TabViews (Tier 3)
    for (size_t i = 0; i < s_num_tabviews; i++) {
        bound_tabview_t *b = &s_tabviews[i];
        if (b->param_id == telem->param_id) {
            uint32_t target_tab = (uint32_t)telem->value.i32_val;
            if (lv_tabview_get_tab_active(b->tabview) != target_tab) {
                b->is_programmatic_update = true;
                lv_tabview_set_active(b->tabview, target_tab, LV_ANIM_OFF);
                b->is_programmatic_update = false;
            }
        }
    }

    // 16. Charts (Tier 3)
    for (size_t i = 0; i < s_num_charts; i++) {
        bound_chart_t *b = &s_charts[i];
        if (b->param_id == telem->param_id && b->series) {
            lv_chart_set_next_value(b->chart, b->series, telem->value.i32_val);
            lv_chart_refresh(b->chart);
        }
    }

    // 17. Images (Phase 1)
    for (size_t i = 0; i < s_num_images; i++) {
        bound_image_t *b = &s_images[i];
        if (b->param_id == telem->param_id && telem->value.str_val[0] != '\0') {
            lv_image_set_src(b->image, telem->value.str_val);
        }
    }

    // 18. ArcLabels (Phase 1)
#if LV_USE_ARCLABEL
    for (size_t i = 0; i < s_num_arclabels; i++) {
        bound_arclabel_t *b = &s_arclabels[i];
        if (b->param_id == telem->param_id) {
            if (telem->value.str_val[0] != '\0') {
                lv_arclabel_set_text(b->arclabel, telem->value.str_val);
            } else if (telem->value.i32_val != 0) {
                lv_arclabel_set_angle_start(b->arclabel, telem->value.i32_val);
            }
        }
    }
#endif

    // 19. QRCodes (Phase 1)
#if LV_USE_QRCODE
    for (size_t i = 0; i < s_num_qrcodes; i++) {
        bound_qrcode_t *b = &s_qrcodes[i];
        if (b->param_id == telem->param_id && telem->value.str_val[0] != '\0') {
            lv_qrcode_set_data(b->qrcode, telem->value.str_val);
        }
    }
#endif

    // 20. Wins (Phase 2)
#if LV_USE_WIN
    for (size_t i = 0; i < s_num_wins; i++) {
        bound_win_t *b = &s_wins[i];
        if (b->param_id == telem->param_id) {
            ui_set_obj_hidden(b->win, !telem->value.bool_val);
        }
    }
#endif

    // 21. ImageButtons (Phase 2)
#if LV_USE_IMAGEBUTTON
    for (size_t i = 0; i < s_num_imgbtns; i++) {
        bound_imagebutton_t *b = &s_imgbtns[i];
        if (b->param_id == telem->param_id && telem->value.str_val[0] != '\0') {
            lv_imagebutton_set_src(b->imgbtn, LV_IMAGEBUTTON_STATE_RELEASED, NULL, telem->value.str_val, NULL);
        }
    }
#endif

    // 22. SpanGroups (Phase 2)
#if LV_USE_SPAN
    for (size_t i = 0; i < s_num_spangroups; i++) {
        bound_spangroup_t *b = &s_spangroups[i];
        if (b->param_id == telem->param_id && telem->value.str_val[0] != '\0') {
            lv_span_t *s = lv_spangroup_get_child(b->spangroup, 0);
            if (s) {
                lv_span_set_text(s, telem->value.str_val);
            }
        }
    }
#endif

    // 23. AnimImages (Phase 2)
#if LV_USE_ANIMIMG
    for (size_t i = 0; i < s_num_animimages; i++) {
        bound_animimage_t *b = &s_animimages[i];
        if (b->param_id == telem->param_id) {
            if (telem->value.bool_val) {
                lv_animimg_start(b->animimg);
            }
        }
    }
#endif

    // 24. Barcodes (Phase 2)
#if LV_USE_BARCODE
    for (size_t i = 0; i < s_num_barcodes; i++) {
        bound_barcode_t *b = &s_barcodes[i];
        if (b->param_id == telem->param_id && telem->value.str_val[0] != '\0') {
            lv_barcode_update(b->barcode, telem->value.str_val);
        }
    }
#endif

    // 25. Dynamic Runtime Widget Flags & States (Phase 3)
    if (telem->param_id == PARAM_WIDGET_FLAGS) {
        param_id_t target_id = (param_id_t)telem->ack_seq_id;
        lv_obj_t *obj = ui_engine_get_widget_by_param(target_id);
        if (obj) {
            uint32_t flags = telem->value.u32_val;
            if (flags & WIDGET_FLAG_HIDDEN) {
                ui_set_obj_hidden(obj, true);
            }
            if (flags & WIDGET_FLAG_VISIBLE) {
                ui_set_obj_hidden(obj, false);
            }
            if (flags & WIDGET_FLAG_DISABLED) {
                lv_obj_add_state(obj, LV_STATE_DISABLED);
            }
            if (flags & WIDGET_FLAG_ENABLED) {
                lv_obj_remove_state(obj, LV_STATE_DISABLED);
            }
            if (flags & WIDGET_FLAG_CLICKABLE) {
                lv_obj_add_flag(obj, LV_OBJ_FLAG_CLICKABLE);
            }
            if (flags & WIDGET_FLAG_UNCLICKABLE) {
                lv_obj_remove_flag(obj, LV_OBJ_FLAG_CLICKABLE);
            }
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
// Declarative Schema Builder (builds Tier 1, 2 & 3 from Rust schema)
// ----------------------------------------------------------------------
void ui_engine_build_from_schema(const widget_descriptor_t *schema, size_t count) {
    if (!schema || count == 0) return;
    s_num_lookups = 0;
    lv_obj_t *scr = lv_screen_active();

    for (size_t i = 0; i < count; i++) {
        const widget_descriptor_t *d = &schema[i];
        lv_align_t align = map_align(d->align);

        // Resolve container parent
        lv_obj_t *parent_obj = scr;
        if (d->parent_param_id != PARAM_NONE) {
            // Check if parent is a TabView
            for (size_t t = 0; t < s_num_tabviews; t++) {
                if (s_tabviews[t].param_id == d->parent_param_id) {
                    if (d->tab_index < s_tabviews[t].num_tabs) {
                        parent_obj = s_tabviews[t].tabs[d->tab_index];
                    }
                    break;
                }
            }
            if (parent_obj == scr && d->widget_type != WIDGET_TYPE_KEYBOARD) {
                lv_obj_t *found = ui_engine_get_widget_by_param(d->parent_param_id);
                if (found) {
#if LV_USE_WIN
                    for (size_t w = 0; w < s_num_wins; w++) {
                        if (s_wins[w].param_id == d->parent_param_id) {
                            found = s_wins[w].content;
                            break;
                        }
                    }
#endif
                    parent_obj = found;
                }
            }
        }

        switch (d->widget_type) {
        // --- Tier 1 Widgets ---
        case WIDGET_TYPE_SWITCH: {
            lv_obj_t *sw = lv_switch_create(parent_obj);
            lv_obj_align(sw, align, d->offset_x, d->offset_y);

            lv_obj_t *lbl = lv_label_create(parent_obj);
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
            lv_obj_t *lbl = lv_label_create(parent_obj);
            lv_obj_align(lbl, align, d->offset_x, d->offset_y - 40);

            lv_obj_t *slider = lv_slider_create(parent_obj);
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
            lv_obj_t *btn = lv_button_create(parent_obj);
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
            lv_obj_t *lbl = lv_label_create(parent_obj);
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
            lv_obj_t *arc = lv_arc_create(parent_obj);
            lv_obj_align(arc, align, d->offset_x, d->offset_y);
            lv_arc_set_range(arc, d->min_val, d->max_val);
            lv_arc_set_value(arc, d->initial_val);

            lv_obj_t *lbl = NULL;
            if (d->label_secondary) {
                lbl = lv_label_create(parent_obj);
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
            lv_obj_t *bar = lv_bar_create(parent_obj);
            lv_obj_align(bar, align, d->offset_x, d->offset_y);
            lv_bar_set_range(bar, d->min_val, d->max_val);
            lv_bar_set_value(bar, d->initial_val, LV_ANIM_OFF);

            lv_obj_t *lbl = NULL;
            if (d->label_primary) {
                lbl = lv_label_create(parent_obj);
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
            lv_obj_t *cb = lv_checkbox_create(parent_obj);
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
            lv_obj_t *led = lv_led_create(parent_obj);
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
            lv_obj_t *sp = lv_spinner_create(parent_obj);
            lv_obj_set_size(sp, 40, 40);
            lv_obj_align(sp, align, d->offset_x, d->offset_y);
            if (!d->initial_val) {
                ui_set_obj_hidden(sp, true);
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
            lv_obj_t *dd = lv_dropdown_create(parent_obj);
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
                b->is_programmatic_update = false;

                lv_obj_add_event_cb(dd, on_dropdown_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, dd);
            break;
        }

        case WIDGET_TYPE_ROLLER: {
            lv_obj_t *roller = lv_roller_create(parent_obj);
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
                b->is_programmatic_update = false;

                lv_obj_add_event_cb(roller, on_roller_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, roller);
            break;
        }

        case WIDGET_TYPE_SPINBOX: {
            lv_obj_t *sb = lv_spinbox_create(parent_obj);
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
                b->is_programmatic_update = false;

                lv_obj_add_event_cb(sb, on_spinbox_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, sb);
            break;
        }

        // --- Tier 2: Text, Keypads, Modals & Gauges ---
        case WIDGET_TYPE_TEXTAREA: {
            lv_obj_t *ta = lv_textarea_create(parent_obj);
            lv_obj_align(ta, align, d->offset_x, d->offset_y);
            lv_obj_set_size(ta, d->width > 0 ? d->width : 220, d->height > 0 ? d->height : 45);
            if (d->label_primary) {
                lv_textarea_set_placeholder_text(ta, d->label_primary);
            }
            if (d->label_secondary) {
                lv_textarea_set_text(ta, d->label_secondary);
            }
            if (d->min_val == 1) {
                lv_textarea_set_one_line(ta, true);
            }
            if (d->max_val > 0) {
                lv_textarea_set_max_length(ta, (uint32_t)d->max_val);
            }

            lv_obj_add_event_cb(ta, on_textarea_focused, LV_EVENT_FOCUSED, NULL);
            lv_obj_add_event_cb(ta, on_textarea_focused, LV_EVENT_CLICKED, NULL);

            if (s_num_textareas < MAX_BINDINGS) {
                bound_textarea_t *b = &s_textareas[s_num_textareas++];
                b->param_id = d->param_id;
                b->textarea = ta;
                b->current_seq = 0;

                lv_obj_add_event_cb(ta, on_textarea_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, ta);
            break;
        }

        case WIDGET_TYPE_KEYBOARD: {
            lv_obj_t *kb = lv_keyboard_create(parent_obj);
            lv_obj_align(kb, align, d->offset_x, d->offset_y);
            if (d->width > 0 && d->height > 0) {
                lv_obj_set_size(kb, d->width, d->height);
            }

            param_id_t target_id = PARAM_NONE;
            if (d->initial_val > 0) {
                target_id = (param_id_t)d->initial_val;
            } else if (d->parent_param_id != PARAM_NONE && parent_obj == scr) {
                target_id = d->parent_param_id;
            }
            if (target_id != PARAM_NONE) {
                lv_obj_t *target_ta = ui_engine_get_widget_by_param(target_id);
                if (target_ta) {
                    lv_keyboard_set_textarea(kb, target_ta);
                }
            }

            if (s_num_keyboards < MAX_BINDINGS) {
                bound_keyboard_t *b = &s_keyboards[s_num_keyboards++];
                b->param_id = d->param_id;
                b->keyboard = kb;
                b->current_seq = 0;

                lv_obj_add_event_cb(kb, on_keyboard_event, LV_EVENT_ALL, b);
            }
            register_lookup(d->param_id, kb);
            break;
        }

        case WIDGET_TYPE_BUTTON_MATRIX: {
            lv_obj_t *bm = lv_buttonmatrix_create(parent_obj);
            lv_obj_align(bm, align, d->offset_x, d->offset_y);
            lv_obj_set_size(bm, d->width > 0 ? d->width : 220, d->height > 0 ? d->height : 140);

            if (s_num_btnmatrices < MAX_BINDINGS) {
                const char **map = parse_btnmatrix_map(s_num_btnmatrices, d->label_primary);
                if (map) {
                    lv_buttonmatrix_set_map(bm, map);
                }

                bound_btnmatrix_t *b = &s_btnmatrices[s_num_btnmatrices++];
                b->param_id = d->param_id;
                b->btnmatrix = bm;
                b->current_seq = 0;

                lv_obj_add_event_cb(bm, on_btnmatrix_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, bm);
            break;
        }

        case WIDGET_TYPE_MSGBOX: {
            lv_obj_t *mbox = lv_msgbox_create(parent_obj);
            if (d->label_primary) {
                lv_msgbox_add_title(mbox, d->label_primary);
            }
            if (d->label_secondary) {
                lv_msgbox_add_text(mbox, d->label_secondary);
            }
            lv_obj_t *close_btn = lv_msgbox_add_close_button(mbox);
            lv_obj_t *ok_btn = lv_msgbox_add_footer_button(mbox, "OK");

            if (d->initial_val == 0) {
                ui_set_obj_hidden(mbox, true);
            }

            if (s_num_msgboxes < MAX_BINDINGS) {
                bound_msgbox_t *b = &s_msgboxes[s_num_msgboxes++];
                b->param_id = d->param_id;
                b->msgbox = mbox;
                b->current_seq = 0;

                if (close_btn) lv_obj_add_event_cb(close_btn, on_msgbox_close_clicked, LV_EVENT_CLICKED, b);
                if (ok_btn) lv_obj_add_event_cb(ok_btn, on_msgbox_ok_clicked, LV_EVENT_CLICKED, b);
            }
            register_lookup(d->param_id, mbox);
            break;
        }

        case WIDGET_TYPE_SCALE: {
            lv_obj_t *scale = lv_scale_create(parent_obj);
            lv_scale_set_mode(scale, LV_SCALE_MODE_ROUND_INNER);
            lv_obj_set_size(scale, d->width > 0 ? d->width : 160, d->height > 0 ? d->height : 160);
            lv_obj_align(scale, align, d->offset_x, d->offset_y);
            lv_scale_set_range(scale, d->min_val, d->max_val);
            lv_scale_set_angle_range(scale, 240);
            lv_scale_set_rotation(scale, 150);
            lv_scale_set_total_tick_count(scale, 21);
            lv_scale_set_major_tick_every(scale, 5);
            lv_scale_set_label_show(scale, true);

            lv_obj_t *needle = lv_line_create(scale);
            lv_scale_set_line_needle_value(scale, needle, -25, d->initial_val);

            if (s_num_scales < MAX_BINDINGS) {
                bound_scale_t *b = &s_scales[s_num_scales++];
                b->param_id = d->param_id;
                b->scale = scale;
                b->needle = needle;
            }
            register_lookup(d->param_id, scale);
            break;
        }

        // --- Tier 3: Containers, Navigation & Data Streams ---
        case WIDGET_TYPE_TABVIEW: {
            lv_obj_t *tv = lv_tabview_create(parent_obj);
            lv_tabview_set_tab_bar_position(tv, LV_DIR_TOP);
            lv_obj_set_size(tv, d->width > 0 ? d->width : LV_PCT(100), d->height > 0 ? d->height : LV_PCT(100));
            lv_obj_align(tv, align, d->offset_x, d->offset_y);

            if (s_num_tabviews < MAX_BINDINGS) {
                bound_tabview_t *b = &s_tabviews[s_num_tabviews++];
                b->param_id = d->param_id;
                b->tabview = tv;
                b->num_tabs = 0;
                b->current_seq = 0;
                b->is_programmatic_update = false;

                // Parse tab names separated by \n
                if (d->label_primary) {
                    char buf[128];
                    strncpy(buf, d->label_primary, sizeof(buf) - 1);
                    buf[sizeof(buf) - 1] = '\0';
                    char *tok = strtok(buf, "\n");
                    while (tok && b->num_tabs < 8) {
                        lv_obj_t *tab = lv_tabview_add_tab(tv, tok);
#if LV_USE_FLEX
                        apply_layout_flow(tab, d->layout_flow);
#endif
                        b->tabs[b->num_tabs++] = tab;
                        tok = strtok(NULL, "\n");
                    }
                }

                lv_obj_add_event_cb(tv, on_tabview_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, tv);
            break;
        }

        case WIDGET_TYPE_TILEVIEW: {
            lv_obj_t *tv = lv_tileview_create(parent_obj);
            lv_obj_set_size(tv, d->width > 0 ? d->width : LV_PCT(100), d->height > 0 ? d->height : LV_PCT(100));
            lv_obj_align(tv, align, d->offset_x, d->offset_y);

            if (s_num_tileviews < MAX_BINDINGS) {
                bound_tileview_t *b = &s_tileviews[s_num_tileviews++];
                b->param_id = d->param_id;
                b->tileview = tv;
                b->num_tiles = 0;

                lv_obj_t *t1 = lv_tileview_add_tile(tv, 0, 0, LV_DIR_RIGHT);
                lv_obj_t *lbl1 = lv_label_create(t1);
                lv_label_set_text(lbl1, "TileView (0,0): Swipe Right ->");
                lv_obj_center(lbl1);

                lv_obj_t *t2 = lv_tileview_add_tile(tv, 1, 0, LV_DIR_LEFT);
                lv_obj_t *lbl2 = lv_label_create(t2);
                lv_label_set_text(lbl2, "<- TileView (1,0): Swipe Left");
                lv_obj_center(lbl2);

                b->tiles[0] = t1;
                b->tiles[1] = t2;
                b->num_tiles = 2;
            }
            register_lookup(d->param_id, tv);
            break;
        }

        case WIDGET_TYPE_MENU: {
            lv_obj_t *menu = lv_menu_create(parent_obj);
            lv_obj_set_size(menu, d->width > 0 ? d->width : 220, d->height > 0 ? d->height : 220);
            lv_obj_align(menu, align, d->offset_x, d->offset_y);

            lv_obj_t *page = lv_menu_page_create(menu, d->label_primary ? d->label_primary : "Menu");
            lv_obj_t *cont1 = lv_menu_cont_create(page);
            lv_obj_t *lbl1 = lv_label_create(cont1);
            lv_label_set_text(lbl1, "WiFi: Connected (STA)");

            lv_obj_t *cont2 = lv_menu_cont_create(page);
            lv_obj_t *lbl2 = lv_label_create(cont2);
            lv_label_set_text(lbl2, "Display: 480x480 RGB565");

            lv_obj_t *cont3 = lv_menu_cont_create(page);
            lv_obj_t *lbl3 = lv_label_create(cont3);
            lv_label_set_text(lbl3, "Core Engine: FreeRTOS + Rust");

            lv_menu_set_page(menu, page);

            if (s_num_menus < MAX_BINDINGS) {
                bound_menu_t *b = &s_menus[s_num_menus++];
                b->param_id = d->param_id;
                b->menu = menu;
            }
            register_lookup(d->param_id, menu);
            break;
        }

        case WIDGET_TYPE_LIST: {
            lv_obj_t *list = lv_list_create(parent_obj);
            lv_obj_set_size(list, d->width > 0 ? d->width : 180, d->height > 0 ? d->height : 160);
            lv_obj_align(list, align, d->offset_x, d->offset_y);

            if (d->label_primary) {
                char buf[128];
                strncpy(buf, d->label_primary, sizeof(buf) - 1);
                buf[sizeof(buf) - 1] = '\0';
                char *tok = strtok(buf, "\n");
                while (tok) {
                    lv_obj_t *btn = lv_list_add_button(list, NULL, tok);
                    lv_obj_add_event_cb(btn, on_list_button_clicked, LV_EVENT_CLICKED, (void *)(intptr_t)d->param_id);
                    tok = strtok(NULL, "\n");
                }
            }

            if (s_num_lists < MAX_BINDINGS) {
                bound_list_t *b = &s_lists[s_num_lists++];
                b->param_id = d->param_id;
                b->list = list;
            }
            register_lookup(d->param_id, list);
            break;
        }

        case WIDGET_TYPE_CHART: {
            lv_obj_t *chart = lv_chart_create(parent_obj);
            lv_obj_set_size(chart, d->width > 0 ? d->width : 220, d->height > 0 ? d->height : 120);
            lv_obj_align(chart, align, d->offset_x, d->offset_y);
            lv_chart_set_type(chart, LV_CHART_TYPE_LINE);
            lv_chart_set_point_count(chart, 16);
            lv_chart_set_range(chart, LV_CHART_AXIS_PRIMARY_Y, d->min_val, d->max_val);

            lv_chart_series_t *ser = lv_chart_add_series(chart, lv_palette_main(LV_PALETTE_BLUE), LV_CHART_AXIS_PRIMARY_Y);

            if (s_num_charts < MAX_BINDINGS) {
                bound_chart_t *b = &s_charts[s_num_charts++];
                b->param_id = d->param_id;
                b->chart = chart;
                b->series = ser;
            }
            register_lookup(d->param_id, chart);
            break;
        }

        case WIDGET_TYPE_TABLE: {
            lv_obj_t *table = lv_table_create(parent_obj);
            lv_obj_set_size(table, d->width > 0 ? d->width : 220, d->height > 0 ? d->height : 120);
            lv_obj_align(table, align, d->offset_x, d->offset_y);

            uint32_t rows = (uint32_t)((d->min_val >> 16) & 0xFFFF);
            if (rows == 0) rows = 3;
            uint32_t cols = (uint32_t)(d->min_val & 0xFFFF);
            if (cols == 0) cols = 2;

            lv_table_set_row_count(table, rows);
            lv_table_set_column_count(table, cols);

            if (d->label_primary) {
                char buf[128];
                strncpy(buf, d->label_primary, sizeof(buf) - 1);
                buf[sizeof(buf) - 1] = '\0';
                char *tok = strtok(buf, ";\n");
                uint32_t r = 0, c = 0;
                while (tok && r < rows) {
                    lv_table_set_cell_value(table, r, c, tok);
                    c++;
                    if (c >= cols) { c = 0; r++; }
                    tok = strtok(NULL, ";\n");
                }
            }

            if (s_num_tables < MAX_BINDINGS) {
                bound_table_t *b = &s_tables[s_num_tables++];
                b->param_id = d->param_id;
                b->table = table;
            }
            register_lookup(d->param_id, table);
            break;
        }

        case WIDGET_TYPE_CALENDAR: {
            lv_obj_t *cal = lv_calendar_create(parent_obj);
            lv_obj_set_size(cal, d->width > 0 ? d->width : 220, d->height > 0 ? d->height : 220);
            lv_obj_align(cal, align, d->offset_x, d->offset_y);
            lv_calendar_set_today_date(cal, 2026, 10, 11);
            lv_calendar_set_month_shown(cal, 2026, 10);
            lv_calendar_header_arrow_create(cal);

            if (s_num_calendars < MAX_BINDINGS) {
                bound_calendar_t *b = &s_calendars[s_num_calendars++];
                b->param_id = d->param_id;
                b->calendar = cal;

                lv_obj_add_event_cb(cal, on_calendar_event_value_changed, LV_EVENT_VALUE_CHANGED, b);
            }
            register_lookup(d->param_id, cal);
            break;
        }

        case WIDGET_TYPE_CANVAS: {
            lv_obj_t *cv = lv_canvas_create(parent_obj);
            uint32_t w = d->width > 0 ? (uint32_t)d->width : 100;
            uint32_t h = d->height > 0 ? (uint32_t)d->height : 100;
            lv_obj_set_size(cv, w, h);
            lv_obj_align(cv, align, d->offset_x, d->offset_y);

            if (s_num_canvases < 2) {
                size_t c_idx = s_num_canvases;
                lv_canvas_set_buffer(cv, s_canvas_buffers[c_idx], w, h, LV_COLOR_FORMAT_RGB565);
                lv_canvas_fill_bg(cv, lv_palette_lighten(LV_PALETTE_GREY, 3), LV_OPA_COVER);

                // Draw test pattern: border and cross
                for (uint32_t y = 5; y < 95 && y < h; y++) {
                    for (uint32_t x = 5; x < 95 && x < w; x++) {
                        if (x == y || x == (99 - y) || x == 5 || x == 94 || y == 5 || y == 94) {
                            lv_canvas_set_px(cv, (int32_t)x, (int32_t)y, lv_palette_main(LV_PALETTE_TEAL), LV_OPA_COVER);
                        }
                    }
                }

                bound_canvas_t *b = &s_canvases[s_num_canvases++];
                b->param_id = d->param_id;
                b->canvas = cv;
                b->cbuf = s_canvas_buffers[c_idx];
            }
            register_lookup(d->param_id, cv);
            break;
        }

        // --- Phase 1: Visuals & IoT Core ---
        case WIDGET_TYPE_IMAGE: {
            lv_obj_t *img = lv_image_create(parent_obj);
            if (d->label_primary) {
                lv_image_set_src(img, d->label_primary);
            }
            if (d->width > 0 && d->height > 0) {
                lv_obj_set_size(img, d->width, d->height);
            }
            lv_obj_align(img, align, d->offset_x, d->offset_y);

            if (s_num_images < MAX_BINDINGS) {
                bound_image_t *b = &s_images[s_num_images++];
                b->param_id = d->param_id;
                b->image = img;
            }
            register_lookup(d->param_id, img);
            break;
        }

        case WIDGET_TYPE_LINE: {
            lv_obj_t *line = lv_line_create(parent_obj);
            lv_obj_align(line, align, d->offset_x, d->offset_y);
            lv_obj_set_style_line_width(line, 4, 0);
            lv_obj_set_style_line_color(line, lv_palette_main(LV_PALETTE_TEAL), 0);
            lv_obj_set_style_line_rounded(line, true, 0);

            if (s_num_lines < MAX_BINDINGS) {
                bound_line_t *b = &s_lines[s_num_lines++];
                b->param_id = d->param_id;
                b->line = line;
                b->point_cnt = 0;

                if (d->label_primary) {
                    b->point_cnt = parse_line_points(d->label_primary, b->points, 16);
                }
                if (b->point_cnt == 0) {
                    int32_t w = d->width > 0 ? d->width : 100;
                    int32_t h = d->height > 0 ? d->height : 20;
                    b->points[0].x = 0;     b->points[0].y = h;
                    b->points[1].x = w / 2; b->points[1].y = 0;
                    b->points[2].x = w;     b->points[2].y = h;
                    b->point_cnt = 3;
                }
                lv_line_set_points(line, b->points, b->point_cnt);
            }
            register_lookup(d->param_id, line);
            break;
        }

        case WIDGET_TYPE_ARCLABEL: {
#if LV_USE_ARCLABEL
            lv_obj_t *al = lv_arclabel_create(parent_obj);
            if (d->label_primary) {
                lv_arclabel_set_text(al, d->label_primary);
            }
            uint32_t radius = d->initial_val > 0 ? (uint32_t)d->initial_val : 70;
            int32_t sz = (int32_t)(radius * 2 + 40);
            lv_obj_set_size(al, d->width > 0 ? d->width : sz, d->height > 0 ? d->height : sz);
            lv_arclabel_set_radius(al, radius);
            if (d->min_val != 0 || d->max_val != 0) {
                lv_arclabel_set_angle_start(al, d->min_val);
                lv_arclabel_set_angle_size(al, d->max_val > 0 ? d->max_val : 180);
            }
            lv_obj_align(al, align, d->offset_x, d->offset_y);

            if (s_num_arclabels < MAX_BINDINGS) {
                bound_arclabel_t *b = &s_arclabels[s_num_arclabels++];
                b->param_id = d->param_id;
                b->arclabel = al;
            }
            register_lookup(d->param_id, al);
#endif
            break;
        }

        case WIDGET_TYPE_QRCODE: {
#if LV_USE_QRCODE
            int32_t size = d->width > 0 ? d->width : 120;
            lv_obj_t *qr = lv_qrcode_create(parent_obj);
            lv_qrcode_set_size(qr, size);
            lv_qrcode_set_dark_color(qr, lv_color_black());
            lv_qrcode_set_light_color(qr, lv_color_white());
            if (d->label_primary) {
                lv_qrcode_set_data(qr, d->label_primary);
            }
            lv_obj_align(qr, align, d->offset_x, d->offset_y);

            if (s_num_qrcodes < MAX_BINDINGS) {
                bound_qrcode_t *b = &s_qrcodes[s_num_qrcodes++];
                b->param_id = d->param_id;
                b->qrcode = qr;
            }
            register_lookup(d->param_id, qr);
#endif
            break;
        }

        // --- Phase 2: Complex UI & Rich Media ---
        case WIDGET_TYPE_WIN: {
#if LV_USE_WIN
            lv_obj_t *win = lv_win_create(parent_obj);
            if (d->width > 0 && d->height > 0) {
                lv_obj_set_size(win, d->width, d->height);
            }
            lv_obj_align(win, align, d->offset_x, d->offset_y);
            if (d->label_primary) {
                lv_win_add_title(win, d->label_primary);
            }
            lv_obj_t *close_btn = lv_win_add_button(win, LV_SYMBOL_CLOSE, 30);
            if (d->initial_val == 0) {
                ui_set_obj_hidden(win, true);
            }
            lv_obj_t *content = lv_win_get_content(win);
#if LV_USE_FLEX
            apply_layout_flow(content, d->layout_flow);
#endif

            if (s_num_wins < MAX_BINDINGS) {
                bound_win_t *b = &s_wins[s_num_wins++];
                b->param_id = d->param_id;
                b->win = win;
                b->content = content;
                if (close_btn) {
                    lv_obj_add_event_cb(close_btn, on_win_close_clicked, LV_EVENT_CLICKED, b);
                }
            }
            register_lookup(d->param_id, win);
#endif
            break;
        }

        case WIDGET_TYPE_IMAGEBUTTON: {
#if LV_USE_IMAGEBUTTON
            lv_obj_t *ib = lv_imagebutton_create(parent_obj);
            lv_obj_align(ib, align, d->offset_x, d->offset_y);
            if (d->label_primary) {
                lv_imagebutton_set_src(ib, LV_IMAGEBUTTON_STATE_RELEASED, NULL, d->label_primary, NULL);
            }

            if (s_num_imgbtns < MAX_BINDINGS) {
                bound_imagebutton_t *b = &s_imgbtns[s_num_imgbtns++];
                b->param_id = d->param_id;
                b->imgbtn = ib;
                b->current_seq = 0;
                b->command_val = (uint32_t)d->initial_val;

                lv_obj_add_event_cb(ib, on_imgbtn_event_clicked, LV_EVENT_CLICKED, b);
            }
            register_lookup(d->param_id, ib);
#endif
            break;
        }

        case WIDGET_TYPE_SPAN_GROUP: {
#if LV_USE_SPAN
            lv_obj_t *sg = lv_spangroup_create(parent_obj);
            if (d->width > 0 && d->height > 0) {
                lv_obj_set_size(sg, d->width, d->height);
            }
            lv_obj_align(sg, align, d->offset_x, d->offset_y);

            if (d->label_primary) {
                char buf[256];
                strncpy(buf, d->label_primary, sizeof(buf) - 1);
                buf[sizeof(buf) - 1] = '\0';
                char *tok = strtok(buf, "|\n");
                uint32_t span_idx = 0;
                while (tok) {
                    lv_span_t *s = lv_spangroup_add_span(sg);
                    lv_span_set_text(s, tok);
                    if (span_idx % 2 == 1) {
                        lv_style_set_text_color(lv_span_get_style(s), lv_palette_main(LV_PALETTE_TEAL));
                    }
                    tok = strtok(NULL, "|\n");
                    span_idx++;
                }
            }

            if (s_num_spangroups < MAX_BINDINGS) {
                bound_spangroup_t *b = &s_spangroups[s_num_spangroups++];
                b->param_id = d->param_id;
                b->spangroup = sg;
            }
            register_lookup(d->param_id, sg);
#endif
            break;
        }

        case WIDGET_TYPE_ANIMIMAGE: {
#if LV_USE_ANIMIMG
            lv_obj_t *ai = lv_animimg_create(parent_obj);
            lv_obj_align(ai, align, d->offset_x, d->offset_y);
            static const void *s_default_anim_imgs[] = {
                LV_SYMBOL_REFRESH,
                LV_SYMBOL_SETTINGS,
                LV_SYMBOL_WIFI,
                LV_SYMBOL_OK,
            };
            lv_animimg_set_src(ai, s_default_anim_imgs, 4);
            uint32_t dur = d->initial_val > 0 ? (uint32_t)d->initial_val : 1200;
            lv_animimg_set_duration(ai, dur);
            lv_animimg_set_repeat_count(ai, LV_ANIM_REPEAT_INFINITE);
            lv_animimg_start(ai);

            if (s_num_animimages < MAX_BINDINGS) {
                bound_animimage_t *b = &s_animimages[s_num_animimages++];
                b->param_id = d->param_id;
                b->animimg = ai;
            }
            register_lookup(d->param_id, ai);
#endif
            break;
        }

        case WIDGET_TYPE_BARCODE: {
#if LV_USE_BARCODE
            int32_t w = d->width > 0 ? d->width : 150;
            int32_t h = d->height > 0 ? d->height : 50;
            lv_obj_t *bc = lv_barcode_create(parent_obj);
            lv_obj_set_size(bc, w, h);
            lv_barcode_set_dark_color(bc, lv_color_black());
            lv_barcode_set_light_color(bc, lv_color_white());
            if (d->label_primary) {
                lv_barcode_update(bc, d->label_primary);
            }
            lv_obj_align(bc, align, d->offset_x, d->offset_y);

            if (s_num_barcodes < MAX_BINDINGS) {
                bound_barcode_t *b = &s_barcodes[s_num_barcodes++];
                b->param_id = d->param_id;
                b->barcode = bc;
            }
            register_lookup(d->param_id, bc);
#endif
            break;
        }

        default:
            break;
        }

#if LV_USE_FLEX
        if (d->layout_flow != LAYOUT_FLOW_NONE) {
            lv_obj_t *created_obj = ui_engine_get_widget_by_param(d->param_id);
            if (created_obj) {
                apply_layout_flow(created_obj, d->layout_flow);
            }
        }
#endif
    }
}
