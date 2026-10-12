# AGENTS.md — AI Agent Guidance & Architectural Grounding

> **Target Audience:** Autonomous AI coding agents (Claude, Gemini, ChatGPT/Codex, etc.) working in this repository.  
> **Mandatory Reading:** Before implementing features, refactoring code, or debugging issues in this repository, you **MUST** read [`DEVELOPING.md`](DEVELOPING.md) and adhere to the architectural rules described below.

---

## 1. Golden Rules for AI Agents

1. **Read `DEVELOPING.md` First:**
   - [`DEVELOPING.md`](DEVELOPING.md) is the primary developer and contributor guide. It details repo layout, build commands, test workflows, coding conventions, and common pitfalls.
   - You must consult [`DEVELOPING.md`](DEVELOPING.md) whenever you are asked to build, test, modify, or add components to this project.

2. **Respect the Core Architectural Boundary:**
   > **"C owns the display; Rust owns the system."**
   - **C (`components/esp_lvgl_bridge`)**: Drives hardware peripherals, FreeRTOS tasks, display buffers, touch controllers, and the LVGL v9 scene graph.
   - **Rust (`crates/esp32-rust-lvgl-bridge-core`, application crates)**: Implements deterministic domain logic, closed-loop state machines, and declarative UI schemas as a bare-metal `no_std` static library (`staticlib`).
   - **Zero Raw Pointers Crossing Boundary**: Never pass raw `lv_obj_t*` pointers, heap-allocated references, or complex Rust types across the FFI boundary. All communication flows through Plain Old Data (POD) structs (`UiCommand`, `DeviceTelemetry`) across FreeRTOS queues or in-memory test rings.

3. **Maintain Preprocessor Guards for Optional Widgets:**
   - When modifying `components/esp_lvgl_bridge/src/ui_binding.c`, ensure that any LVGL widget types requiring specific features (e.g. `LV_USE_ARCLABEL`, `LV_USE_QRCODE`, `LV_USE_WIN`, `LV_USE_IMAGEBUTTON`, `LV_USE_SPAN`, `LV_USE_ANIMIMG`, `LV_USE_BARCODE`, `LV_USE_FLEX`) are properly guarded by `#if LV_USE_*` macros across structs, static storage arrays, callback handlers, and schema constructors.
   - This prevents `-Wunused-variable` or missing symbol errors when building with minimal `sdkconfig` profiles.

4. **Verify Across Both Host and Target Suites:**
   - **Host Fast Verification:** Always run `cargo test --workspace` (or `make test`) before finishing a task. All 54+ tests (including headless visual tests) must pass with zero failures.
   - **Target Build Verification:** When touching C code or build configs, run `make build-component-tests` (or `make build-everything`) using the ESP-IDF environment to guarantee zero compiler warnings and successful linking.

5. **Zero-Warning Policy:**
   - Maintain zero compiler warnings across GCC/Clang on host and `xtensa-esp32s3-elf-gcc` on ESP-IDF.
   - Suppress legitimate deprecation warnings in test suites or legacy LVGL migration shims using targeted `#pragma GCC diagnostic push / ignored / pop` blocks rather than globally disabling warnings.

---

## 2. Repository Navigation Cheat Sheet

| Directory / File | Description | Primary Language |
|---|---|---|
| [`DEVELOPING.md`](DEVELOPING.md) | Contributor and developer manual (builds, tests, coding rules) | Markdown |
| [`README.md`](README.md) | User-facing documentation for engineers using the framework in their apps | Markdown |
| [`Makefile`](Makefile) | Canonical shortcuts (`make test`, `make build-everything`, `make docs`) | Make |
| [`crates/esp32-rust-lvgl-bridge-core/`](crates/esp32-rust-lvgl-bridge-core) | Core ABI contracts, POD types, and declarative widget constructors | Rust (`no_std`) |
| [`crates/esp32-rust-lvgl-bridge-harness/`](crates/esp32-rust-lvgl-bridge-harness) | Headless LVGL host test harness for offscreen rendering & input simulation | Rust + C |
| [`components/esp_lvgl_bridge/`](components/esp_lvgl_bridge) | ESP-IDF component for FreeRTOS queue transports & LVGL schema binding | C |
| [`components/bsp_guition_s3_4848s040/`](components/bsp_guition_s3_4848s040) | Board support package for Guition 4.0" 480×480 ST7701S RGB + GT911 touch | C |
| [`examples/minimal/`](examples/minimal) | Minimal 3-widget starter application template | C + Rust |
| [`examples/everything/`](examples/everything) | Full 34-widget reference application across 7 tabs | C + Rust |
| [`test_apps/component_tests/`](test_apps/component_tests) | On-target ESP-IDF Unity unit test application | C |
| [`docs/`](docs) | mdBook technical documentation source | Markdown |

---

## 3. Standard Agent Workflows

### Task: Modifying C Bridge or UI Binding (`ui_binding.c`)
1. Edit [`components/esp_lvgl_bridge/src/ui_binding.c`](components/esp_lvgl_bridge/src/ui_binding.c).
2. Check macro guards around new static storage or widget bindings (`#if LV_USE_*`).
3. Add corresponding target unit tests in [`components/esp_lvgl_bridge/test/test_ui_binding.c`](components/esp_lvgl_bridge/test/test_ui_binding.c).
4. Add corresponding host unit tests in [`crates/esp32-rust-lvgl-bridge-harness/tests/c_bridge_components_test.rs`](crates/esp32-rust-lvgl-bridge-harness/tests/c_bridge_components_test.rs).
5. Run `cargo test --workspace` to verify host FFI behavior.
6. Run `make build-component-tests` in an ESP-IDF environment to ensure target compilation and zero warnings.

### Task: Adding a New Widget or Schema Descriptor
1. Add new variant in [`crates/esp32-rust-lvgl-bridge-core/src/abi.rs`](crates/esp32-rust-lvgl-bridge-core/src/abi.rs) (`WidgetType` and constructor methods on `WidgetDescriptor`).
2. Add corresponding `widget_type_t` enum and constructor branch in [`components/esp_lvgl_bridge/include/bridge_protocol.h`](components/esp_lvgl_bridge/include/bridge_protocol.h) and [`components/esp_lvgl_bridge/src/ui_binding.c`](components/esp_lvgl_bridge/src/ui_binding.c).
3. If new layout options are added (e.g. Flex Container layout flows), map them via `apply_layout_flow`.
4. Update [`examples/everything/crates/everything-app/src/lib.rs`](examples/everything/crates/everything-app/src/lib.rs) and the widget suite test in [`examples/everything/crates/everything-app/tests/widget_suite_test.rs`](examples/everything/crates/everything-app/tests/widget_suite_test.rs).
5. Run `make test` and `make docs` to update documentation and test artifacts.

### Task: Debugging Compilation or Link Failures
- **Python / IDF Export Issue:**
  Ensure Python is exported properly:
  ```bash
  export PATH="$HOME/.local/share/mise/installs/python/3.13.12/bin:$PATH"
  source $HOME/esp/esp-idf/export.sh
  ```
- **Rust Xtensa Target:**
  When compiling bare-metal static libraries for ESP32-S3:
  ```bash
  cargo build --release --target xtensa-esp32s3-none-elf -p everything-app
  ```
  Note that `cargo test` runs natively on the host target (`x86_64` or `aarch64`) and does not require Xtensa LLVM.

---

## 4. Key Conventions
- **Commit Messages**: Follow Conventional Commits format (`feat(...)`, `fix(...)`, `test(...)`, `docs(...)`, `refactor(...)`).
- **Path References**: When referring to project files in responses, use clickable markdown links (e.g. `[ui_binding.c](file:///abs/path/ui_binding.c)`).
- **Preserve Documentation**: Do not remove existing docstrings or architecture comments when refactoring.
