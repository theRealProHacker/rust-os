all: run

ARM_TOOLS = /home/mi/linnert/arm/bin/
QEMU = $(ARM_TOOLS)qemu-bsprak
BINARY = rust_os/prebuilt/rust-os-3
LINKER_PATH = /usr/local/lib:/import/sage-7.4/local/lib/

run:
	export LD_LIBRARY_PATH=$(LINKER_PATH) && $(QEMU) -kernel $(BINARY)
