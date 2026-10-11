# esp_rust.cmake - Helper function for building and linking Rust static libraries into ESP-IDF components
#
# Usage:
#   include(${ESP_LVGL_BRIDGE_DIR}/cmake/esp_rust.cmake)
#   esp_rust_link_staticlib(
#       COMPONENT ${COMPONENT_LIB}
#       CRATE_NAME "app-core"
#       CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/.."
#   )

function(esp_rust_link_staticlib)
    cmake_parse_arguments(
        RUST_ARG
        ""
        "COMPONENT;CRATE_NAME;CRATE_DIR"
        ""
        ${ARGN}
    )

    if(NOT RUST_ARG_COMPONENT)
        message(FATAL_ERROR "esp_rust_link_staticlib: COMPONENT is required")
    endif()
    if(NOT RUST_ARG_CRATE_NAME)
        message(FATAL_ERROR "esp_rust_link_staticlib: CRATE_NAME is required")
    endif()
    if(NOT RUST_ARG_CRATE_DIR)
        set(RUST_ARG_CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/..")
    endif()

    if(IDF_TARGET STREQUAL "esp32s3")
        set(RUST_TARGET "xtensa-esp32s3-none-elf")
    elseif(IDF_TARGET STREQUAL "esp32c6")
        set(RUST_TARGET "riscv32imac-unknown-none-elf")
    elseif(IDF_TARGET STREQUAL "esp32p4")
        set(RUST_TARGET "riscv32imafc-unknown-none-elf")
    elseif(IDF_TARGET STREQUAL "esp32")
        set(RUST_TARGET "xtensa-esp32-none-elf")
    else()
        set(RUST_TARGET "riscv32imafc-unknown-none-elf")
    endif()

    string(REPLACE "-" "_" RUST_LIB_FILENAME "${RUST_ARG_CRATE_NAME}")
    set(RUST_LIB_PATH "${RUST_ARG_CRATE_DIR}/target/${RUST_TARGET}/release/lib${RUST_LIB_FILENAME}.a")

    set(RUST_TARGET_NAME "cargo_${RUST_LIB_FILENAME}")
    add_custom_target(
        ${RUST_TARGET_NAME}
        ALL
        COMMAND cargo build --release --target ${RUST_TARGET} -Zbuild-std=core,alloc -p ${RUST_ARG_CRATE_NAME}
        WORKING_DIRECTORY "${RUST_ARG_CRATE_DIR}"
        BYPRODUCTS "${RUST_LIB_PATH}"
        COMMENT "Compiling Rust crate '${RUST_ARG_CRATE_NAME}' for target ${RUST_TARGET}..."
    )

    add_dependencies(${RUST_ARG_COMPONENT} ${RUST_TARGET_NAME})
    target_link_libraries(${RUST_ARG_COMPONENT} PRIVATE "${RUST_LIB_PATH}")
endfunction()
