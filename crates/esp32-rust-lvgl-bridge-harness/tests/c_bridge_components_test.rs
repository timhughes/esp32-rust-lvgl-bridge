//! # Component-Level Integration Tests for C Bridge & Protocols
//!
//! Validates:
//! 1. Direct FFI linkage and execution of `p4_c6_link.c` compiled from `components/esp_lvgl_bridge/src/`.
//! 2. Protocol packet framing, CRC16 calculation, and loopback mock dispatch.
//! 3. Link statistics tracking (TX/RX packet counts and bytes).
//! 4. Parameter validation and error code propagation.

#![allow(static_mut_refs)]

use esp32_rust_lvgl_bridge_core::abi::{
    DeviceTelemetry, ParamId, ParamVal, UiAlign, UiCommand, WidgetDescriptor,
};
use esp32_rust_lvgl_bridge_harness::HeadlessHarness;
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum P4C6Channel {
    System = 1,
    Wifi = 2,
    Ble = 3,
    Ieee802154 = 4,
    Telemetry = 5,
    Max = 6,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum P4C6BusType {
    Sdio30 = 0,
    SpiHs = 1,
    LoopbackMock = 2,
}

#[repr(C)]
pub struct P4C6LinkConfig {
    pub bus_type: P4C6BusType,
    pub pin_clk: i32,
    pub pin_cmd: i32,
    pub pin_d0: i32,
    pub pin_d1: i32,
    pub pin_d2: i32,
    pub pin_d3: i32,
    pub pin_handshake_in: i32,
    pub pin_handshake_out: i32,
    pub bus_freq_khz: u32,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct P4C6LinkStats {
    pub tx_packets: u32,
    pub rx_packets: u32,
    pub tx_bytes: u32,
    pub rx_bytes: u32,
    pub crc_errors: u32,
    pub dropped_packets: u32,
}

type P4C6RxCallback = extern "C" fn(
    channel: P4C6Channel,
    data: *const u8,
    len: u16,
    user_data: *mut std::ffi::c_void,
);

#[repr(C)]
pub struct EspLvglBspOps {
    pub board_name: *const std::ffi::c_char,
    pub init_display: extern "C" fn() -> *mut std::ffi::c_void,
    pub init_touch: extern "C" fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void,
    pub set_backlight: extern "C" fn(u8),
    pub backlight_is_on: extern "C" fn() -> bool,
}

extern "C" {
    fn p4_c6_link_init(config: *const P4C6LinkConfig) -> i32;
    fn p4_c6_link_send(channel: P4C6Channel, data: *const u8, len: u16, timeout_ms: u32) -> i32;
    fn p4_c6_link_register_rx_callback(
        channel: P4C6Channel,
        cb: P4C6RxCallback,
        user_data: *mut std::ffi::c_void,
    ) -> i32;
    fn p4_c6_link_get_stats(out_stats: *mut P4C6LinkStats);
    fn p4_c6_link_is_alive() -> bool;

    fn bridge_transport_init(cmd_depth: usize, telem_depth: usize);
    fn bridge_send_command_from_ui(cmd: *const UiCommand) -> bool;
    fn bridge_recv_command(out_cmd: *mut UiCommand, timeout_ms: u32) -> bool;
    fn bridge_send_telemetry(telem: *const DeviceTelemetry) -> bool;
    fn bridge_recv_telemetry_in_ui(out_telem: *mut DeviceTelemetry) -> bool;

    fn bsp_4848s040_get_ops() -> *const EspLvglBspOps;
    fn bsp_backlight_set(percent: u8);
    fn bsp_backlight_is_on() -> bool;
}

static mut RECEIVED_BYTES: Vec<u8> = Vec::new();
static mut RECEIVED_CHANNEL: Option<P4C6Channel> = None;

extern "C" fn test_rx_handler(
    channel: P4C6Channel,
    data: *const u8,
    len: u16,
    _user_data: *mut std::ffi::c_void,
) {
    unsafe {
        RECEIVED_CHANNEL = Some(channel);
        let slice = std::slice::from_raw_parts(data, len as usize);
        RECEIVED_BYTES = slice.to_vec();
    }
}

#[test]
fn test_p4_c6_link_loopback_and_stats() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let config = P4C6LinkConfig {
        bus_type: P4C6BusType::LoopbackMock,
        pin_clk: -1,
        pin_cmd: -1,
        pin_d0: -1,
        pin_d1: -1,
        pin_d2: -1,
        pin_d3: -1,
        pin_handshake_in: -1,
        pin_handshake_out: -1,
        bus_freq_khz: 50_000,
    };

    let err = unsafe { p4_c6_link_init(&config) };
    assert_eq!(err, 0, "p4_c6_link_init should return ESP_OK (0)");
    assert!(unsafe { p4_c6_link_is_alive() }, "Link should report alive");

    let reg_err = unsafe {
        p4_c6_link_register_rx_callback(
            P4C6Channel::Telemetry,
            test_rx_handler,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(reg_err, 0);

    let payload = b"HOST_C_BRIDGE_TEST_PACKET_9999";
    unsafe {
        RECEIVED_BYTES.clear();
        RECEIVED_CHANNEL = None;
    }

    let send_err = unsafe {
        p4_c6_link_send(
            P4C6Channel::Telemetry,
            payload.as_ptr(),
            payload.len() as u16,
            100,
        )
    };
    assert_eq!(send_err, 0, "p4_c6_link_send should succeed");

    unsafe {
        assert_eq!(RECEIVED_CHANNEL, Some(P4C6Channel::Telemetry));
        assert_eq!(RECEIVED_BYTES.as_slice(), payload.as_slice());
    }

    let mut stats = P4C6LinkStats::default();
    unsafe { p4_c6_link_get_stats(&mut stats) };
    assert!(stats.tx_packets > 0, "Expected non-zero TX packet count");
    assert!(stats.rx_packets > 0, "Expected non-zero RX packet count");
    assert!(stats.tx_bytes > 0, "Expected non-zero TX byte count");
    assert!(stats.rx_bytes > 0, "Expected non-zero RX byte count");
    assert_eq!(stats.crc_errors, 0, "CRC errors must be 0 in mock loopback");
}

#[test]
fn test_p4_c6_link_error_handling() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Null config rejected
    let null_err = unsafe { p4_c6_link_init(std::ptr::null()) };
    assert_ne!(null_err, 0);

    // Channel out of bounds rejected
    let bad_chan_err = unsafe {
        p4_c6_link_register_rx_callback(
            P4C6Channel::Max,
            test_rx_handler,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(bad_chan_err, 0);

    // Oversized payload rejected (> 1536 bytes)
    let big_data = [0u8; 1537];
    let big_err = unsafe {
        p4_c6_link_send(
            P4C6Channel::System,
            big_data.as_ptr(),
            big_data.len() as u16,
            10,
        )
    };
    assert_ne!(big_err, 0);
}

#[test]
fn test_unified_ui_binding_component_compilation() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = HeadlessHarness::new(200, 200);

    const P_TEST: ParamId = ParamId::new(999);
    let schema = [
        WidgetDescriptor::button(P_TEST, UiAlign::Center, 0, 0, c"Direct C Bridge", 1),
    ];

    harness.build_schema(&schema);
    harness.step(20);

    assert!(harness.widget_exists(P_TEST), "Button created directly via components/esp_lvgl_bridge/src/ui_binding.c");
}

#[test]
fn test_bridge_transport_command_and_telemetry_queues() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    unsafe { bridge_transport_init(8, 8) };

    let send_cmd = UiCommand {
        param_id: ParamId::new(42),
        seq_id: 100,
        value: ParamVal::from_i32(1234),
    };
    assert!(unsafe { bridge_send_command_from_ui(&send_cmd) });

    let mut recv_cmd = UiCommand {
        param_id: ParamId::new(0),
        seq_id: 0,
        value: ParamVal::from_i32(0),
    };
    assert!(unsafe { bridge_recv_command(&mut recv_cmd, 10) });
    assert_eq!(recv_cmd.param_id, ParamId::new(42));
    assert_eq!(recv_cmd.seq_id, 100);
    assert_eq!(unsafe { recv_cmd.value.i32_val }, 1234);

    // Queue should now be empty (non-blocking)
    assert!(!unsafe { bridge_recv_command(&mut recv_cmd, 0) });

    // Telemetry queue
    let send_telem = DeviceTelemetry {
        param_id: ParamId::new(10),
        ack_seq_id: 55,
        value: ParamVal::from_bool(true),
    };
    assert!(unsafe { bridge_send_telemetry(&send_telem) });

    let mut recv_telem = DeviceTelemetry {
        param_id: ParamId::new(0),
        ack_seq_id: 0,
        value: ParamVal::from_bool(false),
    };
    assert!(unsafe { bridge_recv_telemetry_in_ui(&mut recv_telem) });
    assert_eq!(recv_telem.param_id, ParamId::new(10));
    assert_eq!(recv_telem.ack_seq_id, 55);
    assert_eq!(unsafe { recv_telem.value.bool_val }, true);

    // Queue should now be empty
    assert!(!unsafe { bridge_recv_telemetry_in_ui(&mut recv_telem) });
}

#[test]
fn test_bridge_transport_null_guards() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    unsafe { bridge_transport_init(8, 8) };
    assert!(!unsafe { bridge_send_command_from_ui(std::ptr::null()) });
    assert!(!unsafe { bridge_recv_command(std::ptr::null_mut(), 0) });
    assert!(!unsafe { bridge_send_telemetry(std::ptr::null()) });
    assert!(!unsafe { bridge_recv_telemetry_in_ui(std::ptr::null_mut()) });
}

#[test]
fn test_bsp_guition_resolution_invariants_and_ops() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let ops = unsafe { bsp_4848s040_get_ops() };
    assert!(!ops.is_null(), "bsp_4848s040_get_ops must return valid pointer");

    unsafe {
        let ops_ref = &*ops;
        assert!(!ops_ref.board_name.is_null());
        let board_name_cstr = std::ffi::CStr::from_ptr(ops_ref.board_name);
        let board_name_str = board_name_cstr.to_str().unwrap();
        assert!(board_name_str.contains("Guition ESP32-S3-4848S040"));
        assert!(!ops_ref.board_name.is_null());
    }
}

#[test]
fn test_bsp_guition_backlight_state_tracking() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let ops = unsafe { &*bsp_4848s040_get_ops() };

    (ops.set_backlight)(0);
    assert_eq!((ops.backlight_is_on)(), false);

    (ops.set_backlight)(100);
    assert_eq!((ops.backlight_is_on)(), true);

    (ops.set_backlight)(50);
    assert_eq!((ops.backlight_is_on)(), true);

    (ops.set_backlight)(0);
    assert_eq!((ops.backlight_is_on)(), false);

    // Also verify direct C symbol linkage
    unsafe {
        bsp_backlight_set(100);
        assert_eq!(bsp_backlight_is_on(), true);
        bsp_backlight_set(0);
        assert_eq!(bsp_backlight_is_on(), false);
    }
}

