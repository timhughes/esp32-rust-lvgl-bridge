cmake_minimum_required(VERSION 3.16)

message(FATAL_ERROR
    "\n========================================================================\n"
    "  esp32-rust-lvgl-framework is a component framework library.\n\n"
    "  To build a firmware binary, build one of the official examples in examples/:\n"
    "    - make build-minimal     (or: idf.py -C examples/minimal -DIDF_TARGET=esp32s3 build)\n"
    "    - make build-everything  (or: idf.py -C examples/everything -DIDF_TARGET=esp32s3 build)\n"
    "========================================================================\n"
)
