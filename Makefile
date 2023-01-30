all: build run

CARGO_DIR = /home/mi/jbork/.cargo/bin/
CARGO = $(CARGO_DIR)cargo
ARM_TOOLS = /home/mi/linnert/arm/bin/
QEMU = $(ARM_TOOLS)qemu-bsprak
BINARY = target/armv4t-none-eabi/release/rust-os
LINKER_PATH = /usr/local/lib:/import/sage-7.4/local/lib/

build:
	cd rust_os && export PATH="$(ARM_TOOLS):$(CARGO_DIR):$$PATH" && cargo build --release

run:
	cd rust_os && export LD_LIBRARY_PATH=$(LINKER_PATH) && $(QEMU) -kernel $(BINARY)

debug:
	@echo gdb rust_os/$(BINARY)
	@echo target remote localhost:1234
	cd rust_os && export LD_LIBRARY_PATH=$(LINKER_PATH) && $(QEMU) -s -kernel $(BINARY)

clean:
	cd rust_os && $(CARGO) clean

dis:
	cd rust_os && $(CARGO) objdump --release -- -C -l > ../dis.txt