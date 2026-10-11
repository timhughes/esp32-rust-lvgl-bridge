cmake_minimum_required(VERSION 3.16)

# Register framework components
set(EXTRA_COMPONENT_DIRS "${CMAKE_CURRENT_LIST_DIR}/../../components")

include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(esp32-rust-lvgl-minimal)
