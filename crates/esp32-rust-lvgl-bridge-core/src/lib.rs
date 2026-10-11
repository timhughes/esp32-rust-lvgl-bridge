//! # Bridge Core: C / Rust FFI Contracts & Worker Traits
//!
//! This crate defines the shared, zero-allocation Plain Old Data (POD) types,
//! parameter enumerations, and worker traits bridging FreeRTOS C tasks (LVGL UI)
//! with the Rust state engine.
//!
//! ## Core Architecture
//!
//! - **Zero Heap Sharing**: No raw pointers or dynamically allocated objects cross the C/Rust boundary.
//! - **By-Value Message Passing**: Data moves across FreeRTOS queues via fixed-size POD structs:
//!   - [`UiCommand`]: UI interactions from user touch (button presses, slider drags).
//!   - [`DeviceTelemetry`]: State broadcasts from the Rust domain worker.
//! - **Declarative UI Descriptors**: [`WidgetDescriptor`] structures allow
//!   Rust to define UI hierarchies that C constructs without procedural UI code.
//!
//! ## Example
//!
//! ```
//! use esp32_rust_lvgl_bridge_core::{ParamId, UiAlign, WidgetDescriptor, WidgetType};
//!
//! const PARAM_TABS: ParamId = ParamId::new(10);
//! const PARAM_POWER: ParamId = ParamId::new(11);
//!
//! static SCHEMA: &[WidgetDescriptor] = &[
//!     WidgetDescriptor::tabview(PARAM_TABS, c"Main\nSettings", 480, 480),
//!     WidgetDescriptor::switch(PARAM_POWER, UiAlign::TopLeft, 20, 15, c"On", c"Off")
//!         .with_parent(PARAM_TABS, 0),
//! ];
//! assert_eq!(SCHEMA[0].widget_type, WidgetType::TabView);
//! assert_eq!(SCHEMA[1].tab_index, 0);
//! ```
//!
//! ## Modules
//! - [`abi`]: C-compatible data structures, parameter identifiers, and alignment constants.
//! - [`worker`]: Generic traits for closed-loop controllers and worker execution loops.

#![cfg_attr(not(test), no_std)]

pub mod abi;
pub mod worker;

pub use abi::*;
pub use worker::*;
