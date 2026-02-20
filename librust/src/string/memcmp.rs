#[unsafe(no_mangle)]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let b1 = *s1.add(i);
            let b2 = *s2.add(i);
            if b1 != b2 {
                return (b1 as i32) - (b2 as i32);
            }
        }
        return 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    return memcmp(s1, s2, n);
}