#![no_std]

extern crate limine;

use limine::framebuffer_info;
use spin::Mutex;

mod font;

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;

pub static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new());

/// Standard VGA 16-colour palette → 32-bit 0x00RRGGBB.
const VGA_PALETTE: [u32; 16] = [
    0x000000, // 0  Black
    0x0000AA, // 1  Blue
    0x00AA00, // 2  Green
    0x00AAAA, // 3  Cyan
    0xAA0000, // 4  Red
    0xAA00AA, // 5  Magenta
    0xAA5500, // 6  Brown
    0xAAAAAA, // 7  Light grey
    0x555555, // 8  Dark grey
    0x5555FF, // 9  Light blue
    0x55FF55, // 10 Light green
    0x55FFFF, // 11 Light cyan
    0xFF5555, // 12 Light red
    0xFF55FF, // 13 Light magenta
    0xFFFF55, // 14 Yellow
    0xFFFFFF, // 15 White
];

pub struct Terminal {
    row: usize,
    column: usize,
    fg: u32,
    bg: u32,
    cols: usize,
    rows: usize,
}

impl Terminal {
    pub const fn new() -> Self {
        Terminal {
            row: 0,
            column: 0,
            fg: 0xAAAAAA,  // light grey
            bg: 0x000000,  // black
            cols: 0,
            rows: 0,
        }
    }

    /// Lazily compute the text-grid dimensions from the framebuffer.
    fn ensure_dims(&mut self) {
        if self.cols == 0 {
            if let Some(fb) = framebuffer_info() {
                self.cols = fb.width as usize / FONT_WIDTH;
                self.rows = fb.height as usize / FONT_HEIGHT;
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(fb) = framebuffer_info() {
            self.cols = fb.width as usize / FONT_WIDTH;
            self.rows = fb.height as usize / FONT_HEIGHT;

            // Fill the entire framebuffer with the background colour.
            let pixels = fb.height as usize * (fb.pitch as usize / 4);
            let ptr = fb.address as *mut u32;
            for i in 0..pixels {
                unsafe { core::ptr::write_volatile(ptr.add(i), self.bg); }
            }

            self.row = 0;
            self.column = 0;
        }
    }

    /// Set foreground (low nibble) and background (high nibble) from a
    /// VGA-style attribute byte, e.g. `0x0F` = white on black.
    #[unsafe(no_mangle)]
    pub fn set_color(&mut self, color: u8) {
        self.fg = VGA_PALETTE[(color & 0x0F) as usize];
        self.bg = VGA_PALETTE[((color >> 4) & 0x0F) as usize];
    }

    /// Draw character `c` at text-grid position (x, y) with the given
    /// VGA attribute byte.
    #[unsafe(no_mangle)]
    pub fn put_entry_at(&self, c: u8, color: u8, x: usize, y: usize) {
        let fg = VGA_PALETTE[(color & 0x0F) as usize];
        let bg = VGA_PALETTE[((color >> 4) & 0x0F) as usize];
        self.draw_char(c, fg, bg, x, y);
    }

    // ── internal helpers ────────────────────────────────────────────

    /// Blit an 8x8 glyph at text-grid position (col, row).
    fn draw_char(&self, c: u8, fg: u32, bg: u32, col: usize, row: usize) {
        if let Some(fb) = framebuffer_info() {
            let glyph = font::get_glyph(c);
            let px = col * FONT_WIDTH;
            let py = row * FONT_HEIGHT;
            let stride = fb.pitch as usize / 4; // pixels per scanline

            let base = fb.address as *mut u32;
            for gy in 0..FONT_HEIGHT {
                let bits = glyph[gy];
                for gx in 0..FONT_WIDTH {
                    let pixel = if bits & (0x80 >> gx) != 0 { fg } else { bg };
                    let off = (py + gy) * stride + (px + gx);
                    unsafe { core::ptr::write_volatile(base.add(off), pixel); }
                }
            }
        }
    }

    /// Scroll the entire screen up by one text row.
    fn move_up(&mut self) {
        if let Some(fb) = framebuffer_info() {
            let pitch = fb.pitch as usize;
            let base = fb.address as *mut u8;
            let row_bytes = FONT_HEIGHT * pitch;
            let total = (self.rows - 1) * row_bytes;

            // Copy rows 1..n to rows 0..n-1.
            unsafe { core::ptr::copy(base.add(row_bytes), base, total); }

            // Clear the last text row.
            let last = base.wrapping_add((self.rows - 1) * row_bytes) as *mut u32;
            let pixels_in_row = FONT_HEIGHT * (pitch / 4);
            for i in 0..pixels_in_row {
                unsafe { core::ptr::write_volatile(last.add(i), self.bg); }
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn write(&mut self, c: u8) {
        self.ensure_dims();
        if self.cols == 0 {
            return; // framebuffer not ready
        }

        if c == b'\n' {
            self.column = 0;
            self.row += 1;
            if self.row >= self.rows {
                self.move_up();
                self.row = self.rows - 1;
            }
        } else {
            self.draw_char(c, self.fg, self.bg, self.column, self.row);
            self.column += 1;
            if self.column >= self.cols {
                self.column = 0;
                self.row += 1;
                if self.row >= self.rows {
                    self.move_up();
                    self.row = self.rows - 1;
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
