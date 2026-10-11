idf_component_register(
    SRCS "main.c"
    REQUIRES esp_lvgl_bridge bsp_guition_s3_4848s040 lvgl esp_timer
)

# Link Rust widget-cookbook static library using the framework's reusable CMake helper
include(${CMAKE_CURRENT_LIST_DIR}/../../../components/esp_lvgl_bridge/cmake/esp_rust.cmake)
esp_rust_link_staticlib(
    COMPONENT ${COMPONENT_LIB}
    CRATE_NAME "widget-cookbook"
    CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../.."
)
