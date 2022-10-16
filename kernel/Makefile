TARGET := riscv64gc-unknown-none-elf
MODE := debug
ifeq ($(MODE), release)
	BUILD_MODE = --release
endif

KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel

BOOTLOADER := ../bootloader/rustsbi-qemu.bin

# Binary utils
OBJCOPY := rust-objcopy --binary-architecture=riscv64


build: env kernel

kernel:
	@cargo build $(BUILD_MODE)

clean:
	cargo clean

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview


CPUS = 1
QEMU = qemu-system-riscv64
QEMUOPTS += -machine virt -bios none -nographic -kernel $(KERNEL_ELF)
QEMUOPTS += -m 128M -smp $(CPUS)
# QEMUOPTS += -drive file=fs.img,if=none,format=raw,id=x0
# QEMUOPTS += -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

ifeq ($(LAB),net)
QEMUOPTS += -netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap
QEMUOPTS += -device e1000,netdev=net0,bus=pcie.0
endif

qemu: kernel
	$(QEMU) $(QEMUOPTS)


qemu-gdb: kernel
	@echo "*** Now run 'gdb' in another window." 1>&2
	$(QEMU) $(QEMUOPTS) -S -s




.PHONY:

