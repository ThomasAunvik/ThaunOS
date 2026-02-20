#![no_std]
#![feature(c_variadic)]

mod stdlib;
mod stdio;
mod string;

pub use stdlib::*;
pub use stdlib::abort::*;

pub use stdio::*;

pub use string::*;
pub use string::memmove::*;
pub use string::strlen::*;

#[cfg(not(test))]
use crate::stdio::printf::kprintln;
#[cfg(not(test))]
use crate::string::tostring::u32_to_str;

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln(b"Kernel panic!!!");
    if let Some(location) = _info.location() {
        kprintln(b"At:");
        kprintln(location.file().as_bytes());
        kprintln(b":");
        let mut line_buf = [0u8; 12];
        let mut col_buf = [0u8; 12];

        let line = location.line();
        let col = location.column()
        ;
        // Convert line and column numbers to strings without using `alloc` or `core::fmt`.
        let line_str = u32_to_str(line, &mut line_buf);
        kprintln(line_str);
        kprintln(b":");
        let col_str = u32_to_str(col, &mut col_buf);
        kprintln(col_str);
    } else {
        kprintln(b"Unknown location");
    }
    loop {
        core::hint::spin_loop();
    }
}
