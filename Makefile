all: build run

CARGO_DIR = /home/mi/jbork/.cargo/bin/
CARGO = $(CARGO_DIR)cargo
ARM_TOOLS = /home/mi/linnert/arm/bin/
QEMU = $(ARM_TOOLS)qemu-bsprak
BINARY = target/armv4t-none-eabi/debug/rust-os
LINKER_PATH = /usr/local/lib:/import/sage-7.4/local/lib/

build:
	cd rust_os && export PATH="$(ARM_TOOLS):$(CARGO_DIR):$$PATH" && cargo build

run:
	cd rust_os && export LD_LIBRARY_PATH=$(LINKER_PATH) && $(QEMU) -kernel $(BINARY)

debug:
	cd rust_os && export LD_LIBRARY_PATH=$(LINKER_PATH) && $(QEMU) -s -S -kernel $(BINARY)
	@echo gdb /rust_os/$(BINARY)

clean:
	cd rust_os && $(CARGO) clean