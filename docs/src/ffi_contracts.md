# Rust & C FFI Contract Reference

The boundary between C (LVGL) and Rust (`no_std`) relies on Plain Old Data (POD) structures passed strictly by value.

## Parameter IDs
```c
typedef enum {
    PARAM_SETPOINT_HZ    = 1, // i32: Target frequency (Hz)
    PARAM_SYSTEM_ENABLE  = 2, // bool: System run/stop state
    PARAM_TRIGGER_RESET  = 3, // trigger: Momentary reset trigger
    PARAM_CYCLE_COUNT    = 4, // u32: Monotonic execution cycle count
} param_id_t;
```

## Binary Values
```c
typedef union {
    int32_t  i32_val;
    uint32_t u32_val;
    float    f32_val;
    bool     bool_val;
} param_value_t;
```

## Message Structs
```c
typedef struct {
    uint32_t      param_id;
    param_value_t value;
} ui_command_t;

typedef struct {
    uint32_t      param_id;
    param_value_t value;
    uint32_t      timestamp_ms;
} device_telemetry_t;
```

## Exported Rust C Entry Points
* `void rust_worker_init(void)`: Allocates and initializes the deterministic Rust state engine.
* `void rust_worker_step(void)`: Steps the control loop (stepping internal timers, processing pending commands).
* `const widget_descriptor_t *rust_get_s3_schema(size_t *count)`: Returns the static array of UI widget descriptors for the 480×480 screen.
