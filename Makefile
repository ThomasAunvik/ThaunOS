# Toolchain
AS = i686-elf-as
CC = i686-elf-gcc
LD = i686-elf-gcc

# Compiler flags
CFLAGS = -std=gnu99 -ffreestanding -O2 -Wall -Wextra 
LDFLAGS = -ffreestanding -O2 -nostdlib 

# Target
TARGET = thaunos.bin

# Output directory
BUILD_DIR = build
ISO_OUTPUT_DIR = output
ISO_DIR = iso
ISO_FILE = thaunos.iso

# Object files
OBJS = $(BUILD_DIR)/boot.o
BUILD_LIBS = kernel/out/libkernel.a

LIBS = -Lkernel/out -lkernel -lgcc

# Default target
all: $(TARGET)

# Link kernel
$(TARGET): $(OBJS) ${BUILD_LIBS}
	$(LD) -T linker.ld -o $@ $(LDFLAGS) $(OBJS) $(LIBS)

# Compile boot.asm
$(BUILD_DIR)/boot.o: boot.asm
	mkdir -p $(BUILD_DIR)
	$(AS) boot.asm -o $(BUILD_DIR)/boot.o

# Compile kernel.c
# To disable SSE2 instructions, you can add the following flags to the rustc command:
# -C target-feature=-sse,-sse2
kernel/out/libkernel.a: kernel/src/lib.rs
	rustc -C opt-level=2 -C panic=abort --target=i686-unknown-linux-gnu -g --crate-type=staticlib -o kernel/out/libkernel.a ./kernel/src/lib.rs


verify:
	if grub-file --is-x86-multiboot $(TARGET); then echo multiboot confirmed; else echo the file is not multiboot; fi

iso: $(TARGET)
	mkdir -p $(ISO_DIR)/boot/grub
	cp $(TARGET) $(ISO_DIR)/boot/$(TARGET)
	echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo 'menuentry "Thaunos" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '    multiboot /boot/$(TARGET)' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '    boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg

	mkdir -p $(ISO_OUTPUT_DIR)
	grub-mkrescue -o $(ISO_OUTPUT_DIR)/$(ISO_FILE) $(ISO_DIR)

run: iso
	qemu-system-i386 -cdrom $(ISO_OUTPUT_DIR)/$(ISO_FILE)

# Clean build artifacts
clean:
	rm -f $(OBJS) $(TARGET)
	rm -rf $(ISO_DIR) $(ISO_OUTPUT_DIR) $(BUILD_DIR)
	rm -rf kernel/out kernel/target

.PHONY: all verify clean
