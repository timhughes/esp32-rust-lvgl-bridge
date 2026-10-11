cmake_minimum_required(VERSION 3.16)

# Tell ESP-IDF to search root components/ directory
set(EXTRA_COMPONENT_DIRS "${CMAKE_CURRENT_LIST_DIR}/../../components")
set(TEST_COMPONENTS "esp_lvgl_bridge" "bsp_guition_s3_4848s040")

include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(esp32_lvgl_component_tests)
