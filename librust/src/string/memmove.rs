#[unsafe(no_mangle)]
pub extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        if dest < src as *mut u8 {
            for i in 0..n {
                *dest.add(i) = *src.add(i);
            }
        } else {
            for i in (0..n).rev() {
                *dest.add(i) = *src.add(i);
            }
        }
        return dest;
    }
}