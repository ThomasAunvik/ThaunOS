#![no_std]

extern crate tty;

use librust::printf::{ kprintln };
use tty::{ TERMINAL };

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}


#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    {
        let mut term = TERMINAL.lock();
        term.clear();
        term.set_color(0x0F);
    }


    kprintln(b"Hello world!");
    
    kprintln(b"This is a test of the kernel's printf implementation.");
    kprintln(b"This is a test of the kernel's printf implementation2.");
}
