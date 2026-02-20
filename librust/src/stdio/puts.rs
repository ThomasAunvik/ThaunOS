use crate::stdio::printf::printf;

#[unsafe(no_mangle)]
pub extern "C" fn puts(s: *const u8) -> i32 {
    unsafe { 
        return printf(s) 
    };
}