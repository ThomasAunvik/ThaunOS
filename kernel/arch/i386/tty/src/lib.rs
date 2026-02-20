#![no_std]

extern crate vga;

use vga::{ 
    vga_entry_color,
    vga_entry,
    VGA_COLOR_LIGHT_GREY,
    VGA_COLOR_BLACK,
};

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_ADDR: usize = 0xB8000;

pub struct Terminal {
    row: usize,
    column: usize,
    color: u8,
}

/// Write a value to VGA text-mode memory. Hardware I/O requires unsafe.
#[inline(always)]
fn vga_write(index: usize, value: u16) {
    unsafe {
        let buffer = VGA_BUFFER_ADDR as *mut u16;
        core::ptr::write_volatile(buffer.add(index), value);
    }
}

/// Read a value from VGA text-mode memory. Hardware I/O requires unsafe.
#[inline(always)]
fn vga_read(index: usize) -> u16 {
    unsafe {
        let buffer = VGA_BUFFER_ADDR as *const u16;
        core::ptr::read_volatile(buffer.add(index))
    }
}

impl Terminal {
    pub fn new() -> Self {
        let color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
        let entry = vga_entry(b' ', color);
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            vga_write(i, entry);
        }
        Terminal { row: 0, column: 0, color }
    }

    #[unsafe(no_mangle)]
    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }

    #[unsafe(no_mangle)]
    pub fn put_entry_at(&self, c: u8, color: u8, x: usize, y: usize) {
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

    #[unsafe(no_mangle)]
    pub fn scroll(&self, line:  usize) {

        for loop_var in line * (VGA_WIDTH * 2) + VGA_BUFFER_ADDR..(VGA_WIDTH * 2) {
            let c = unsafe { core::ptr::read_volatile(loop_var as *const u8) };
            unsafe { core::ptr::write_volatile((loop_var - (VGA_WIDTH * 2)) as *mut u8, c) };
        }
    }
    
    #[unsafe(no_mangle)]
    pub fn delete_last_line(&self) {
        for x in 0..(VGA_WIDTH*2) {
            let ptr = (VGA_BUFFER_ADDR + (VGA_WIDTH * 2 * (VGA_HEIGHT - 1)) + x) as *mut u8;
            unsafe { core::ptr::write_volatile(ptr, 0) };
        }
    }

    #[unsafe(no_mangle)]
    pub fn write(&mut self, c: u8) {
        if c == b'\n' {
            self.column = 0;
            self.row += 1;
            if self.row == VGA_HEIGHT {
                self.move_up();
                self.row -= 1;
            }
        } else {
            self.put_entry_at(c, self.color, self.column, self.row);
            self.column += 1;
            if self.column == VGA_WIDTH {
                self.column = 0;
                self.row += 1;
                if self.row == VGA_HEIGHT {
                    self.move_up();
                    self.row -= 1;
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn write_string(&mut self, s: &[u8]) {
        for &byte in s {
            if byte == 0 {
                break;
            }
            self.write(byte);
        }
    }
}
