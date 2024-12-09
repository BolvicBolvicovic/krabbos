all: build qemu

build:
	cargo bootimage

qemu:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-krabbos/debug/bootimage-krabbos.bin

debug: build
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-krabbos/debug/bootimage-krabbos.bin -s -S

clean:
	cargo clean

.PHONY: all build qemu clean
