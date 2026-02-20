#[unsafe(no_mangle)]
pub extern "C" fn strlen(s: *const u8) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}