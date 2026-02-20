use tty::{ TERMINAL };

#[unsafe(no_mangle)]
pub extern "C" fn putchar(ic: u8) -> u8 {
    {
        let mut term = TERMINAL.lock();
        term.write(ic);
    }
    return ic;
}