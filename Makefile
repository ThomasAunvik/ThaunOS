
# Target
TARGET = kernel/thaunos.kernel

# Output directory
BUILD_DIR = build
ISO_OUTPUT_DIR = output
ISO_DIR = sysroot
ISO_FILE = thaunos.iso

# Default target
all: $(TARGET) kernel

kernel:
	cd kernel && $(MAKE)

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
	rm -rf $(ISO_DIR) $(ISO_OUTPUT_DIR) $(BUILD_DIR)

.PHONY: all kernel verify iso run clean
