#![no_std]
extern crate tty;

use tty::{ Terminal};

#[unsafe(no_mangle)]
pub fn kernel_main() {
    let mut terminal = Terminal::new();
    terminal.write_string(b"Hello, kernel World!\n");
    terminal.write_string(b"Hello, This Works!!\n");
    terminal.write_string(b"Hello, This Works!!\n");

}
