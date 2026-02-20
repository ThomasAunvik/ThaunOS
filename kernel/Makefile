DEFAULT_HOST=i386-elf
HOST=i386-elf
HOSTARCH=i386
RUST_TARGET=i686-unknown-linux-gnu

CFLAGS?=-O2 -g
LDFLAGS?=
CPPFLAGS?=
LDFLAGS?=
LIBS?=

AS = i686-elf-as
CC = i686-elf-gcc
LD = i686-elf-gcc

RC=rustc
RC_OPTIONS=-C opt-level=2 -C panic=abort --target=$(RUST_TARGET) -g --crate-type=staticlib

DESTDIR?=../iso/
PREFIX?=/usr/local
EXEC_PREFIX?=$(PREFIX)
BOOTDIR?=$(EXEC_PREFIX)/boot
INCLUDEDIR?=$(PREFIX)/include

CFLAGS:=$(CFLAGS) -ffreestanding -Wall -Wextra
CPPFLAGS:=$(CPPFLAGS) -D__is_kernel -Iinclude
LDFLAGS:=$(LDFLAGS) -ffreestanding -O2 -nostdlib
LIBS:=$(LIBS) -nostdlib -lgcc

ARCHDIR=arch/$(HOSTARCH)

include $(ARCHDIR)/make.config

CFLAGS:=$(CFLAGS) $(KERNEL_ARCH_CFLAGS)
CPPFLAGS:=$(CPPFLAGS) $(KERNEL_ARCH_CPPFLAGS)
LDFLAGS:=$(LDFLAGS) $(KERNEL_ARCH_LDFLAGS)
LIBS:=$(LIBS) $(KERNEL_ARCH_LIBS)

KERNEL_OBJS=\
$(KERNEL_ARCH_OBJS)

KERNEL_RUST_OBJS=\
$(KERNEL_ARCH_RUST_OBJS) \
target/$(RUST_TARGET)/release/libkernel.a \
../librust/target/$(RUST_TARGET)/release/liblibrust.a

KERNEL_RUST_PROJECTS=\
$(KERNEL_ARCH_RUST_PROJECTS) \
kernel/Cargo.toml

OBJS=\
$(ARCHDIR)/crti.o \
$(ARCHDIR)/crtbegin.o \
$(KERNEL_OBJS) \
$(ARCHDIR)/crtend.o \
$(ARCHDIR)/crtn.o \

LINK_LIST=\
$(LDFLAGS) \
$(ARCHDIR)/crti.o \
$(ARCHDIR)/crtbegin.o \
$(KERNEL_OBJS) \
$(KERNEL_RUST_OBJS) \
$(LIBS) \
$(ARCHDIR)/crtend.o \
$(ARCHDIR)/crtn.o \

.PHONY: all clean install install-headers install-kernel
.SUFFIXES: .o .a .c .asm .rs

all: thaunos.kernel

thaunos.kernel: $(OBJS) build $(ARCHDIR)/linker.ld
	$(LD) -T $(ARCHDIR)/linker.ld -o $@ $(LDFLAGS) $(LINK_LIST)
	grub-file --is-x86-multiboot thaunos.kernel

$(ARCHDIR)/crtbegin.o $(ARCHDIR)/crtend.o:
	OBJ=`$(CC) $(CFLAGS) $(LDFLAGS) -print-file-name=$(@F)` && cp "$$OBJ" $@

.c.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(CFLAGS) $(CPPFLAGS)

build:
	cargo build --release --target=$(RUST_TARGET)
#$(RC) ${RC_OPTIONS} -o $@ $(shell dirname $@ | cut -d'/' -f-3)/src/lib.rs

.asm.o:
	$(AS) $< -o $@

clean:
	rm -f thaunos.kernel
	rm -f $(OBJS) *.o */*.o */*/*.o
	rm -f $(OBJS:.o=.d) *.d */*.d */*/*.d
	rm -rf target

install: install-headers install-kernel

install-headers:
	mkdir -p $(DESTDIR)$(INCLUDEDIR)
	cp -R --preserve=timestamps include/. $(DESTDIR)$(INCLUDEDIR)/.

install-kernel: thaunos.kernel
	mkdir -p $(DESTDIR)$(BOOTDIR)
	cp thaunos.kernel $(DESTDIR)$(BOOTDIR)

-include $(OBJS:.o=.d)