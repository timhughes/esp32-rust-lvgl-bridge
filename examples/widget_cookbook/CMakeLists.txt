cmake_minimum_required(VERSION 3.16)

# Register reusable framework components from root
set(EXTRA_COMPONENT_DIRS "../../components")

include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(widget-cookbook-example)
