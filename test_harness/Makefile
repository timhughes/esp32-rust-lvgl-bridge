CC ?= gcc
CFLAGS ?= -O2 -g -Wall -Wextra -Wno-unused-function -Wno-unused-parameter -Wno-deprecated-declarations

# Discover LVGL location: check managed components, Espressif cache, or local ./lvgl
LVGL_DIR ?= $(shell \
    if [ -d "../managed_components/lvgl__lvgl" ]; then echo "../managed_components/lvgl__lvgl"; \
    elif [ -d "$$HOME/.cache/Espressif/ComponentManager/service_d92d8f1e/lvgl__lvgl_9.4.0_17e68bfd" ]; then echo "$$HOME/.cache/Espressif/ComponentManager/service_d92d8f1e/lvgl__lvgl_9.4.0_17e68bfd"; \
    elif [ -d "./lvgl" ]; then echo "./lvgl"; \
    else echo "./lvgl"; fi)

RUST_ROOT = ..
RUST_LIB = $(RUST_ROOT)/target/release/libapp_core.a

INCLUDES = -I. \
           -I../main/include \
           -I$(LVGL_DIR) \
           -I$(LVGL_DIR)/src \
           -DLV_CONF_INCLUDE_SIMPLE

SRCS = main_test.c \
       harness.c \
       host_transport.c \
       ../main/ui_binding.c \
       ../main/p4_c6_link.c \
       ../main/main.c \
       $(shell find $(LVGL_DIR)/src -name "*.c" 2>/dev/null)

OBJS = $(SRCS:.c=.o)
TARGET = test_runner

all: $(TARGET)

$(LVGL_DIR):
	@echo "Fetching LVGL v9 for host harness..."
	git clone --depth 1 --branch release/v9.2 https://github.com/lvgl/lvgl.git $(LVGL_DIR)

$(RUST_LIB): FORCE
	@echo "Building Rust app-core staticlib..."
	cd $(RUST_ROOT) && cargo build --release -p app-core

%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

$(TARGET): $(RUST_LIB) $(OBJS)
	$(CC) $(OBJS) $(RUST_LIB) -lm -lpthread -ldl -o $@

run: $(TARGET)
	@mkdir -p artifacts
	./$(TARGET)
	@echo "Snapshot generation complete. Artifacts:"
	@ls -lh artifacts/*.png

clean:
	rm -f $(OBJS) $(TARGET)
	rm -rf artifacts

.PHONY: all run clean FORCE
FORCE:
