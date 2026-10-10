idf_component_register(
    SRCS "main.c" "bridge_transport.c" "ui_binding.c" "p4_c6_link.c" "bsp_4848s040.c"
    INCLUDE_DIRS "include"
    REQUIRES lvgl esp_lcd espressif__esp_lcd_st7701 espressif__esp_lcd_touch_gt911 espressif__esp_lcd_panel_io_additions driver esp_timer
)

if(IDF_TARGET STREQUAL "esp32s3")
    set(RUST_TARGET "xtensa-esp32s3-none-elf")
elseif(IDF_TARGET STREQUAL "esp32c6")
    set(RUST_TARGET "riscv32imac-unknown-none-elf")
elseif(IDF_TARGET STREQUAL "esp32p4")
    set(RUST_TARGET "riscv32imafc-unknown-none-elf")
else()
    set(RUST_TARGET "riscv32imafc-unknown-none-elf")
endif()

set(RUST_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../crates/app-core")
set(RUST_LIB "${CMAKE_CURRENT_SOURCE_DIR}/../target/${RUST_TARGET}/release/libapp_core.a")

add_custom_command(
    OUTPUT "${RUST_LIB}"
    COMMAND cargo build --release --target ${RUST_TARGET} -p app-core
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/.."
    COMMENT "Compiling Rust core staticlib for ${RUST_TARGET}..."
)

add_custom_target(cargo_app_core DEPENDS "${RUST_LIB}")
add_dependencies(${COMPONENT_LIB} cargo_app_core)
target_link_libraries(${COMPONENT_LIB} PRIVATE "${RUST_LIB}")
