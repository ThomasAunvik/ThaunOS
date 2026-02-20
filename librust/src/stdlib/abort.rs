use crate::printf::printf;

pub fn abort() -> ! {
    unsafe {
        printf(b"Aborting...\n\0".as_ptr() as *const u8);
        core::arch::asm!("hlt");
    }

    loop {} /* `loop {}` or `panic!("...")` */
}