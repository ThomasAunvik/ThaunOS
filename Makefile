
# Target
TARGET = kernel/thaunos-x86_64.kernel

# Default architecture is i386, can be overridden by setting ARCH environment variable
ARCH ?= x86_64#i386

# Output directory
BUILD_DIR = build
ISO_OUTPUT_DIR = output
ISO_DIR = sysroot
ISO_FILE = thaunos.iso

LIBS = \
../librust/target/$(ARCH)-unknown-linux-gnu/release/liblibrust.a

# Default target
all: $(TARGET) kernel

kernel: $(KERNEL_LIBS) $(LIBS) kernel-$(ARCH)

kernel/thaunos.kernel:$(LIBS) kernel-i386
kernel/thaunos-x86_64.kernel: $(LIBS) kernel-x86_64 

kernel-i386: $(KERNEL_LIBS)
	cd kernel && $(MAKE)

kernel-x86_64: $(KERNEL_LIBS)
	cd kernel && $(MAKE) -f GNUMakefile-x86_64

../librust/target/x86_64-unknown-linux-gnu/release/liblibrust.a:
	cd librust && cargo build --release --target=$(ARCH)-unknown-linux-gnu

verify: verify-$(ARCH)

verify-i386: $(TARGET)
	if grub-file --is-x86-multiboot $(TARGET); then echo multiboot confirmed; else echo the file is not multiboot; fi

verify-x86_64: $(TARGET)
	if grub-file --is-x86_64-efi $(TARGET); then echo multiboot confirmed; else echo the file is not multiboot; fi

iso: iso-$(ARCH) 

iso-i386: $(TARGET) kernel
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

iso-x86_64: $(TARGET) kernel ./limine ./limine.conf
	mkdir -p $(ISO_OUTPUT_DIR)
	mkdir -p $(ISO_DIR)/boot/limine
	cp kernel/thaunos-x86_64.kernel $(ISO_DIR)/boot/$(notdir $(TARGET))

	cp -v limine.conf \
		limine/limine-bios.sys \
		limine/limine-bios-cd.bin \
		limine/limine-uefi-cd.bin \
		$(ISO_DIR)/boot/limine/

	mkdir -p $(ISO_DIR)/EFI/BOOT
	cp -v limine/BOOTX64.EFI $(ISO_DIR)/EFI/BOOT/
	cp -v limine/BOOTIA32.EFI $(ISO_DIR)/EFI/BOOT/

	xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        $(ISO_DIR) -o $(ISO_OUTPUT_DIR)/$(ISO_FILE)
	
	./limine/limine bios-install $(ISO_OUTPUT_DIR)/$(ISO_FILE) --quiet

disk: disk-$(ARCH)
disk-i386: iso-i386
	dd if=$(ISO_OUTPUT_DIR)/$(ISO_FILE) of=thaunos.img bs=4M status=progress && sync

disk-x86_64: iso-x86_64
	# Create an empty zeroed-out 64MiB image file.
	dd if=/dev/zero bs=1M count=0 seek=64 of=image.hdd

	# Create a partition table.
	PATH=$PATH:/usr/sbin:/sbin sgdisk image.hdd -n 1:2048 -t 1:ef00 -m 1

	# Install the Limine BIOS stages onto the image.
	./limine/limine bios-install image.hdd

	# Format the image as fat32.
	mformat -i image.hdd@@1M

	# Make relevant subdirectories.
	mmd -i image.hdd@@1M ::/EFI ::/EFI/BOOT ::/boot ::/boot/limine

	# Copy over the relevant files.
	mcopy -i image.hdd@@1M bin/myos ::/boot
	mcopy -i image.hdd@@1M limine.conf limine/limine-bios.sys ::/boot/limine
	mcopy -i image.hdd@@1M limine/BOOTX64.EFI ::/EFI/BOOT
	mcopy -i image.hdd@@1M limine/BOOTIA32.EFI ::/EFI/BOOT


./limine:
	git clone https://codeberg.org/Limine/Limine.git limine --branch=v10.x-binary --depth=1
	cd limine && $(MAKE)

run: iso
	qemu-system-$(ARCH) -cdrom $(ISO_OUTPUT_DIR)/$(ISO_FILE) -d int,cpu_reset -D output/err.log -serial file:output/serial.log

# Clean build artifacts
clean:
	cd kernel && $(MAKE) clean
	cd librust && $(MAKE) clean
	rm -rf $(ISO_DIR) $(ISO_OUTPUT_DIR) $(BUILD_DIR)

packages:
	sudo apt install -y build-essential libclang-dev xorriso qemu-system

.PHONY: \
	all \
	kernel \
	kernel-i386 \
	kernel-x86_64 \
	verify \
	iso \
	iso-i386 \
	iso-x86_64 \
	run \
	clean \
	packages
