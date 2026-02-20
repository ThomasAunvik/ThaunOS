#![no_std]

#[cfg(target_arch = "x86")]
extern crate tty_i386;

#[cfg(target_arch = "x86_64")]
extern crate tty_x86_64;

use librust::printf::{ kprintln, kprint };
#[cfg(target_arch = "x86")]
use tty_i386::{ TERMINAL };

#[cfg(target_arch = "x86_64")]
use limine::{ init as init_x86_64, keyboard };

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

// x86_64: assembly entry point that enables SSE before entering Rust.
// The CPU may have SSE disabled; the Rust x86_64 ABI requires it.
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".global kernel_main",
    "kernel_main:",
    // Enable SSE (mirrors i386/boot.asm logic for 64-bit mode)
    // 1) Clear CR0.EM (bit 2), set CR0.MP (bit 1)
    "mov rax, cr0",
    "and eax, 0xFFFFFFFB",
    "or  eax, 0x2",
    "mov cr0, rax",
    // 2) Set CR4.OSFXSR (bit 9) and CR4.OSXMMEXCPT (bit 10)
    "mov rax, cr4",
    "or  eax, 0x600",
    "mov cr4, rax",
    // Align stack to 16 bytes before call (SysV ABI requirement).
    // Limine may jmp or call us, so force alignment explicitly.
    "and rsp, -16",
    // Call Rust entry point
    "call rust_kernel_main",
    // Halt loop (should never return)
    "cli",
    "2: hlt",
    "jmp 2b",
);

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub extern "C" fn rust_kernel_main() {
    init_x86_64();

    kprintln(b"Hello, Thaunos! This is the x86_64 kernel.");
    kprintln(b"Keyboard input enabled. Type something:");

    let mut input = [0u8; 256];
    let mut input_len = 0;

    kprint(b"> ");

    // Echo keyboard input to the screen.
    loop {
        let ch = keyboard::read_char();
        if ch == b'\n' || ch == b'\r' {
            kprintln(b"");

            kprint(b"You typed: ");
            kprint(core::str::from_utf8(&input[..input_len]).unwrap_or("<invalid UTF-8>").as_bytes());
            kprintln(b"");
            kprint(b"> ");
            
            input_len = 0;
        } else if ch == 8 {
            // Backspace: could handle cursor, for now just ignore
        } else {
            kprint(&[ch]);
            if input_len < input.len() {
                input[input_len] = ch;
                input_len += 1;
            }
        }
    }
}

// i386: the entry point is set up by boot.asm which enables SSE,
// then calls kernel_main directly.
#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    loop {
        core::hint::spin_loop();
    }
}
