all: build qemu

build:
	cargo bootimage

qemu:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-krabbos/debug/bootimage-krabbos.bin

clean:
	cargo clean

.PHONY: all build qemu clean
