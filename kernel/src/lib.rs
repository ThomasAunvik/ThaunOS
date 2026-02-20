#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

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

#[inline]
fn vga_entry_color(mut fg: vga_color, mut bg: vga_color) -> uint8_t {
    return (fg as u32 | (bg as u32) << 4) as uint8_t;
}
#[inline]
fn vga_entry(mut uc: u8, mut color: uint8_t) -> uint16_t {
    return (uc as u16 as i32
        | (color as u16 as i32) << 8) as uint16_t;
}

fn strlen(mut str: *const i8) -> usize {
    let mut len: size_t = 0;
    unsafe {
        while *str.offset(len as isize) != 0 {
            len = len.wrapping_add(1);
            len;
        }
    }
    return len;
}

pub static mut terminal_row: size_t = 0;
pub static mut terminal_column: size_t = 0;
pub static mut terminal_color: uint8_t = 0;
pub static mut terminal_buffer: *mut uint16_t = {
    0xb8000 as *mut uint16_t
};

fn terminal_initialize() {
    unsafe {
        terminal_row = 0;
        terminal_column = 0;
        terminal_color = vga_entry_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
    }
    let mut y: size_t = 0;
    while y < 25 {
        let mut x: size_t = 0;
        while x < 80 {
                let index: size_t = y
                    .wrapping_mul(80)
                    .wrapping_add(x);
                unsafe {
                *terminal_buffer
                    .offset(
                        index as isize,
                    ) = vga_entry(b' ', terminal_color);
                x = x.wrapping_add(1);
            }
            x;
        }
        y = y.wrapping_add(1);
        y;
    }
}

fn terminal_setcolor(mut color: uint8_t) {
    unsafe {
        terminal_color = color;
    }
}

fn terminal_putentryat(
    mut c: i8,
    mut color: uint8_t,
    mut x: size_t,
    mut y: size_t,
) {
    let index: size_t = y
        .wrapping_mul(80)
        .wrapping_add(x);

    unsafe {
        *terminal_buffer.offset(index as isize) = vga_entry(c as u8, color);
    }
}

fn terminal_moveup() {
    let mut y: size_t = 1;
    while y < 25 {
        let mut x: size_t = 0;
        while x < 80 {
            let from_index: size_t = y
                .wrapping_mul(80)
                .wrapping_add(x);
            let to_index: size_t = y
                .wrapping_sub(1)
                .wrapping_mul(80)
                .wrapping_add(x);

            unsafe {
            *terminal_buffer
                .offset(
                    to_index as isize,
                ) = *terminal_buffer.offset(from_index as isize);
            }
            x = x.wrapping_add(1);
            x;
        }
        y = y.wrapping_add(1);
        y;
    }
    let mut x_0: size_t = 0;
    while x_0 < 80 {
        let index: size_t = ((25 - 1) * 80_usize)
            .wrapping_add(x_0);

        unsafe {
        *terminal_buffer
            .offset(
                index as isize,
            ) = vga_entry(b' ', terminal_color);
        }
        x_0 = x_0.wrapping_add(1);
        x_0;
    }
}
fn terminal_putchar(mut c: i8) {
    unsafe {
    if c as i32 == '\n' as i32 {
        terminal_column = 0;
        terminal_row = terminal_row.wrapping_add(1);
        if terminal_row == 25 {
            terminal_moveup();
            terminal_row = 24;
        }
        return;
    }
    terminal_putentryat(c, terminal_color, terminal_column, terminal_row);
    terminal_column = terminal_column.wrapping_add(1);
    if terminal_column == 80 {
        terminal_column = 0;
        terminal_row = terminal_row.wrapping_add(1);
        if terminal_row == 25 {
            terminal_moveup();
            terminal_row = 24;
        }
    }
}
}
fn terminal_write(
    mut data: *const i8,
    mut size: size_t,
) {
    unsafe {
    let mut i: size_t = 0;
        while i < size {
            terminal_putchar(*data.offset(i as isize));
            i = i.wrapping_add(1);
            i;
        }
    }
}

fn terminal_writestring(mut data: *const i8) {
    terminal_write(data, strlen(data));
}

fn rand(mut seed: i32) -> i32 {
    return seed;
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    terminal_initialize();
    terminal_writestring(
        b"Hello, kernel World!\n\0" as *const u8 as *const i8,
    );
    terminal_writestring(b"Hello, This Works!!\n\0" as *const u8 as *const i8);
    let mut loop_counter: i32 = 0;


    unsafe {
        let mut message: [i8; 26] = *::std::mem::transmute::<
            &[u8; 26],
            &mut [i8; 26],
        >(b"Hello! Thaunos is alive!\n\0");

        loop {
            loop_counter += 1;
            loop_counter;
            let mut i: i32 = 0;
            while (i as usize)
                < ::std::mem::size_of::<[i8; 26]>()
            {
                loop_counter += 1;
                loop_counter;
                let mut rand_value: i32 = rand(loop_counter);
                if message[i as usize] as i32 == '\n' as i32 {
                    terminal_putchar('\n' as i32 as i8);
                } else {
                    terminal_setcolor(
                        vga_entry_color(
                            VGA_COLOR_WHITE,
                            (rand_value % 16) as vga_color,
                        ),
                    );
                    terminal_putchar(message[i as usize]);
                    if i as usize
                        >= ::std::mem::size_of::<[i8; 26]>()
                            .wrapping_sub(1)
                    {
                        i = -1;
                    }
                }
                i += 1;
                i;
            }
        };
    }
}
