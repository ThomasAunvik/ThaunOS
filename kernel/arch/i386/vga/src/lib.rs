#![no_std]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub type VgaColor = u32;

pub const VGA_COLOR_WHITE: VgaColor = 15;
pub const VGA_COLOR_LIGHT_BROWN: VgaColor = 14;
pub const VGA_COLOR_LIGHT_MAGENTA: VgaColor = 13;
pub const VGA_COLOR_LIGHT_RED: VgaColor = 12;
pub const VGA_COLOR_LIGHT_CYAN: VgaColor = 11;
pub const VGA_COLOR_LIGHT_GREEN: VgaColor = 10;
pub const VGA_COLOR_LIGHT_BLUE: VgaColor = 9;
pub const VGA_COLOR_DARK_GREY: VgaColor = 8;
pub const VGA_COLOR_LIGHT_GREY: VgaColor = 7;
pub const VGA_COLOR_BROWN: VgaColor = 6;
pub const VGA_COLOR_MAGENTA: VgaColor = 5;
pub const VGA_COLOR_RED: VgaColor = 4;
pub const VGA_COLOR_CYAN: VgaColor = 3;
pub const VGA_COLOR_GREEN: VgaColor = 2;
pub const VGA_COLOR_BLUE: VgaColor = 1;
pub const VGA_COLOR_BLACK: VgaColor = 0;

#[unsafe(no_mangle)]
pub fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    (fg | bg << 4) as u8
}

#[unsafe(no_mangle)]
pub fn vga_entry(uc: u8, color: u8) -> u16 {
    uc as u16 | (color as u16) << 8
}
