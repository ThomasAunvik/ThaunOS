#![no_std]
#![allow(dead_code, non_camel_case_types)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub type size_t = usize;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type vga_color = u32;

pub const VGA_COLOR_WHITE: vga_color = 15;
pub const VGA_COLOR_LIGHT_BROWN: vga_color = 14;
pub const VGA_COLOR_LIGHT_MAGENTA: vga_color = 13;
pub const VGA_COLOR_LIGHT_RED: vga_color = 12;
pub const VGA_COLOR_LIGHT_CYAN: vga_color = 11;
pub const VGA_COLOR_LIGHT_GREEN: vga_color = 10;
pub const VGA_COLOR_LIGHT_BLUE: vga_color = 9;
pub const VGA_COLOR_DARK_GREY: vga_color = 8;
pub const VGA_COLOR_LIGHT_GREY: vga_color = 7;
pub const VGA_COLOR_BROWN: vga_color = 6;
pub const VGA_COLOR_MAGENTA: vga_color = 5;
pub const VGA_COLOR_RED: vga_color = 4;
pub const VGA_COLOR_CYAN: vga_color = 3;
pub const VGA_COLOR_GREEN: vga_color = 2;
pub const VGA_COLOR_BLUE: vga_color = 1;
pub const VGA_COLOR_BLACK: vga_color = 0;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_ADDR: usize = 0xB8000;

#[inline]
fn vga_entry_color(fg: vga_color, bg: vga_color) -> uint8_t {
    (fg | bg << 4) as uint8_t
}

#[inline]
fn vga_entry(uc: u8, color: uint8_t) -> uint16_t {
    uc as uint16_t | (color as uint16_t) << 8
}

/// Write a value to VGA text-mode memory. Hardware I/O requires unsafe.
#[inline(always)]
fn vga_write(index: usize, value: uint16_t) {
    unsafe {
        let buffer = VGA_BUFFER_ADDR as *mut uint16_t;
        core::ptr::write_volatile(buffer.add(index), value);
    }
}

/// Read a value from VGA text-mode memory. Hardware I/O requires unsafe.
#[inline(always)]
fn vga_read(index: usize) -> uint16_t {
    unsafe {
        let buffer = VGA_BUFFER_ADDR as *const uint16_t;
        core::ptr::read_volatile(buffer.add(index))
    }
}

struct Terminal {
    row: size_t,
    column: size_t,
    color: uint8_t,
}

impl Terminal {
    fn new() -> Self {
        let color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
        let entry = vga_entry(b' ', color);
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            vga_write(i, entry);
        }
        Terminal { row: 0, column: 0, color }
    }

    fn set_color(&mut self, color: uint8_t) {
        self.color = color;
    }

    fn put_entry_at(&self, c: u8, color: uint8_t, x: size_t, y: size_t) {
        vga_write(y * VGA_WIDTH + x, vga_entry(c, color));
    }

    fn move_up(&self) {
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                vga_write((y - 1) * VGA_WIDTH + x, vga_read(y * VGA_WIDTH + x));
            }
        }
        let entry = vga_entry(b' ', self.color);
        for x in 0..VGA_WIDTH {
            vga_write((VGA_HEIGHT - 1) * VGA_WIDTH + x, entry);
        }
    }

    fn putchar(&mut self, c: u8) {
        if c == b'\n' {
            self.column = 0;
            self.row += 1;
            if self.row == VGA_HEIGHT {
                self.move_up();
                self.row = VGA_HEIGHT - 1;
            }
            return;
        }
        self.put_entry_at(c, self.color, self.column, self.row);
        self.column += 1;
        if self.column == VGA_WIDTH {
            self.column = 0;
            self.row += 1;
            if self.row == VGA_HEIGHT {
                self.move_up();
                self.row = VGA_HEIGHT - 1;
            }
        }
    }

    fn write_string(&mut self, s: &[u8]) {
        for &byte in s {
            if byte == 0 {
                break;
            }
            self.putchar(byte);
        }
    }
}

fn rand(seed: i32) -> i32 {
    seed
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    let mut terminal = Terminal::new();
    terminal.write_string(b"\n");
    terminal.write_string(b" _____ _                       ___  ____  \n");
    terminal.write_string(b"|_   _| |__   __ _ _   _ _ __ / _ \\/ ___| \n");
    terminal.write_string(b"  | | | '_ \\ / _` | | | | '_ \\ | | \\___ \\ \n");
    terminal.write_string(b"  | | | | | | (_| | |_| | | | | |_| |___) |\n");
    terminal.write_string(b"  |_| |_| |_|\\__,_|\\__,_|_| |_|\\___/|____/ \n");
    terminal.write_string(b"\n");

    terminal.write_string(b"Hello, kernel World!\n");
    terminal.write_string(b"Hello, This Works!!\n");
}
