# Thaunos

Thaunos is a hobbyist operating system kernel written primarily in Rust with some assembly, targeting both **i386** (x86 32-bit) and **x86_64** (64-bit) architectures. The i386 build boots via GRUB (Multiboot), while the x86_64 build uses the [Limine](https://github.com/limine-bootloader/limine) boot protocol.

## Project Structure

```
thaunos/
├── Makefile                 # Top-level build orchestrator
├── limine.conf              # Limine bootloader configuration (x86_64)
├── kernel/                  # Kernel workspace (Cargo workspace root)
│   ├── Cargo.toml           # Workspace definition
│   ├── Makefile             # i386 kernel build
│   ├── GNUMakefile-x86_64   # x86_64 kernel build
│   ├── kernel/              # Core kernel crate
│   │   └── src/lib.rs       # Entry points (kernel_main / rust_kernel_main)
│   └── arch/
│       ├── i386/
│       │   ├── boot.asm     # Multiboot entry, stack setup, SSE init
│       │   ├── linker.ld    # i386 linker script (entry: _start, base 2M)
│       │   ├── tty/         # VGA text-mode terminal (80×25 @ 0xB8000)
│       │   └── vga/         # VGA colour helpers
│       └── x86_64/
│           ├── linker.ld    # x86_64 linker script (higher-half @ 0xffffffff80000000)
│           ├── limine/      # Limine protocol bindings, IDT, framebuffer init
│           ├── tty/         # Framebuffer terminal (8×16 bitmap font)
│           └── vga/         # VGA colour helpers
├── librust/                 # Freestanding C-runtime-like library in Rust
│   └── src/
│       ├── stdio/           # printf, putchar, puts
│       ├── stdlib/          # abort
│       └── string/          # memcpy, memmove, memset, memcmp, strlen
├── limine/                  # Limine bootloader binaries & deploy tool
├── sysroot/                 # ISO filesystem tree (generated)
└── output/                  # Build outputs (ISO image, logs)
```

### Key Crates

| Crate | Path | Purpose |
|---|---|---|
| `kernel` | `kernel/kernel/` | Core kernel — entry points, architecture dispatch |
| `limine` | `kernel/arch/x86_64/limine/` | Limine protocol bindings, IDT setup, framebuffer extraction |
| `tty-x86_64` | `kernel/arch/x86_64/tty/` | Framebuffer-based terminal with bitmap font rendering |
| `tty-i386` | `kernel/arch/i386/tty/` | Classic VGA text-mode terminal (80×25) |
| `vga-x86_64` | `kernel/arch/x86_64/vga/` | VGA colour constants and entry helpers |
| `vga-i386` | `kernel/arch/i386/vga/` | VGA colour constants and entry helpers |
| `librust` | `librust/` | Freestanding libc-like library (printf, memcpy, etc.) |

## Prerequisites

### Common (both architectures)

- **Rust** (nightly recommended) — with targets `i686-unknown-linux-gnu` and `x86_64-unknown-linux-gnu`
- **GNU Make**
- **QEMU** — `qemu-system-i386` and/or `qemu-system-x86_64` for testing
- **xorriso** — for creating ISO images (x86_64)

### i386 only

- **i686-elf cross-compiler toolchain** — `i686-elf-gcc`, `i686-elf-as`
- **GRUB** — `grub-mkrescue`, `grub-file`

### x86_64 only

- **A host C compiler** (`cc`) and **linker** (`ld`) — the x86_64 build uses the host toolchain
- **NASM** (optional, if architecture-specific assembly is added)
- **Limine** — cloned and built automatically by the Makefile

### Installing Rust targets

```bash
rustup target add i686-unknown-linux-gnu
rustup target add x86_64-unknown-linux-gnu
```

## Building

All build commands are run from the project root directory.

### x86_64 (default)

```bash
# Build the kernel
make kernel

# Build and create a bootable ISO
make iso

# Build, create ISO, and run in QEMU
make run
```

The x86_64 build uses the Limine boot protocol. On first build, the Makefile will automatically clone and build the Limine deploy tool if the `limine/` directory does not already contain it.

The resulting kernel binary is `kernel/thaunos-x86_64.kernel` and the ISO is written to `output/thaunos.iso`.

### i386

```bash
# Build the kernel
make ARCH=i386 kernel

# Build and create a bootable ISO (uses GRUB)
make ARCH=i386 iso

# Build, create ISO, and run in QEMU
make ARCH=i386 run
```

The i386 build uses the Multiboot standard and boots via GRUB. It requires the `i686-elf-gcc` and `i686-elf-as` cross-compilation tools.

The resulting kernel binary is `kernel/thaunos.kernel` and the ISO is written to `output/thaunos.iso`.

### Build Targets Summary

| Target | Description |
|---|---|
| `make kernel` | Build the kernel for the selected `ARCH` |
| `make iso` | Build kernel + create a bootable ISO |
| `make disk` | Build kernel + create a bootable disk image |
| `make run` | Build + ISO + launch in QEMU |
| `make verify` | Verify the kernel binary format (multiboot / EFI) |
| `make clean` | Remove all build artifacts |

### Specifying Architecture

The `ARCH` variable controls which architecture to build. It defaults to `x86_64`:

```bash
make ARCH=x86_64 <target>   # 64-bit build (Limine, default)
make ARCH=i386 <target>      # 32-bit build (GRUB/Multiboot)
```

## Running in QEMU

```bash
# x86_64
make run

# i386
make ARCH=i386 run
```

QEMU is launched with interrupt/CPU-reset logging (`-d int,cpu_reset`) and serial output redirected to `output/serial.log`. Error logs are written to `output/err.log`.

## How It Works

### Boot Flow — x86_64

1. **Limine** loads the kernel ELF at the higher-half address (`0xffffffff80000000`)
2. Limine jumps to `kernel_main` (defined in inline assembly in the kernel crate)
3. The assembly stub enables **SSE** (required by the Rust x86_64 ABI), then calls `rust_kernel_main`
4. `rust_kernel_main` initialises the **IDT**, validates the Limine base revision, extracts the **framebuffer** info, and prints a hello message via the framebuffer terminal

### Boot Flow — i386

1. **GRUB** loads the Multiboot kernel at physical address `2M`
2. `boot.asm` sets up the stack, enables **SSE**, and jumps to `kernel_main`
3. `kernel_main` (Rust) enters the main kernel loop

### librust

A freestanding Rust library providing basic C-runtime functions (`printf`, `putchar`, `puts`, `memcpy`, `memmove`, `memset`, `memcmp`, `strlen`, `abort`). It compiles to a static library and is linked into the final kernel binary. C headers are auto-generated via `cbindgen`.

## License

See individual files for license information.
