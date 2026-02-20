#[unsafe(no_mangle)]
pub extern "C" fn memset(s: *mut u8, c: u8, n: usize) -> *mut u8 {
    unsafe {
        for i in 0..n {
            *s.add(i) = c;
        }
        return s;
    }
}