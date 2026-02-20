
# Target
TARGET = kernel/thaunos.kernel

ARCH = i386

# Output directory
BUILD_DIR = build
ISO_OUTPUT_DIR = output
ISO_DIR = sysroot
ISO_FILE = thaunos.iso

KERNEL_LIBS=librust/target/i686-unknown-linux-gnu/release/liblibrust.a

# Default target
all: $(TARGET) kernel

kernel: librust $(KERNEL_LIBS)
	cd kernel && $(MAKE)

kernel/thaunos.kernel: $(KERNEL_LIBS)
	cd kernel && $(MAKE)

librust/target/i686-unknown-linux-gnu/release/liblibrust.a:
	cd librust && $(MAKE) release

verify:
	if grub-file --is-x86-multiboot $(TARGET); then echo multiboot confirmed; else echo the file is not multiboot; fi

iso: $(TARGET) kernel
	mkdir -p $(ISO_DIR)/boot/grub
	cp $(TARGET) $(ISO_DIR)/boot/$(notdir $(TARGET))
	echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo 'menuentry "Thaunos" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '    multiboot /boot/$(notdir $(TARGET))' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '    boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg

	mkdir -p $(ISO_OUTPUT_DIR)
	grub-mkrescue -o $(ISO_OUTPUT_DIR)/$(ISO_FILE) $(ISO_DIR)

run: iso
	qemu-system-i386 -cdrom $(ISO_OUTPUT_DIR)/$(ISO_FILE)

# Clean build artifacts
clean:
	cd kernel && $(MAKE) clean
	cd librust && $(MAKE) clean
	rm -rf $(ISO_DIR) $(ISO_OUTPUT_DIR) $(BUILD_DIR)

.PHONY: all kernel verify iso run clean
