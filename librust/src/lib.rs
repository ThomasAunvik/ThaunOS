#![no_std]
#![feature(c_variadic)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

mod stdlib;
mod stdio;
mod string;

pub use stdlib::*;
pub use stdlib::abort::*;

pub use stdio::*;

pub use string::*;
pub use string::memmove::*;
pub use string::strlen::*;