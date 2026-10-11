cmake_minimum_required(VERSION 3.16)

message(FATAL_ERROR
    "\n========================================================================\n"
    "  esp32-rust-lvgl-framework is a component framework library.\n\n"
    "  To build a firmware binary, build one of the official examples in examples/:\n"
    "    - make build-s3       (or: idf.py -C examples/s3_4848_flagship -DIDF_TARGET=esp32s3 build)\n"
    "    - make build-minimal  (or: idf.py -C examples/minimal -DIDF_TARGET=esp32s3 build)\n"
    "    - make build-cookbook (or: idf.py -C examples/widget_cookbook -DIDF_TARGET=esp32s3 build)\n"
    "========================================================================\n"
)
