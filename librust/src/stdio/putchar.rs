
#[cfg(target_arch = "x86_64")]
use tty_x86_64::{ TERMINAL };

#[cfg(target_arch = "x86")]
use tty_i386::{ TERMINAL };

#[unsafe(no_mangle)]
pub extern "C" fn putchar(ic: u8) -> u8 {
    {
        let mut term = TERMINAL.lock();
        term.write(ic);
    }
    return ic;
}