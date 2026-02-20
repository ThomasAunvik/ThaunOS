use crate::string::strlen::strlen;
use crate::stdio::putchar::putchar;

/// Write a byte slice to the terminal. Returns `false` on failure.
fn print_bytes(s: &[u8]) -> bool {
    for &byte in s {
        if putchar(byte) == 0 {
            return false;
        }
    }
    true
}

/// Write a null-terminated C string to the terminal. Returns `false` on failure.
#[unsafe(no_mangle)]
pub extern "C" fn print(s: *const u8) -> bool {
    let len = strlen(s);
    let bytes = unsafe { core::slice::from_raw_parts(s, len) };
    print_bytes(bytes)
}

/// Minimal printf implementation supporting `%c`, `%s`, and `%%`.
///
/// # Safety
/// The caller must ensure that variadic arguments match the format specifiers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn printf(format: *const u8, mut args: ...) -> i32 {
    let fmt_len = strlen(format);
    let fmt = unsafe { core::slice::from_raw_parts(format, fmt_len) };

    let mut written: i32 = 0;
    let mut i = 0;

    while i < fmt.len() {
        let maxrem = i32::MAX - written;

        if fmt[i] != b'%' || (i + 1 < fmt.len() && fmt[i + 1] == b'%') {
            // Literal character (or escaped %%)
            if fmt[i] == b'%' {
                i += 1; // skip the first '%'
            }
            let start = i;
            i += 1;
            while i < fmt.len() && fmt[i] != b'%' {
                i += 1;
            }
            let amount = (i - start) as i32;
            if maxrem < amount {
                // TODO: Set errno to EOVERFLOW.
                return -1;
            }
            if !print_bytes(&fmt[start..start + amount as usize]) {
                return -1;
            }
            written += amount;
            continue;
        }

        // We have a '%' format specifier
        let format_begun_at = i;
        i += 1; // skip '%'

        if i >= fmt.len() {
            break;
        }

        match fmt[i] {
            b'c' => {
                i += 1;
                // char promotes to int in C calling convention
                let c = unsafe { args.arg::<i32>() } as u8;
                if maxrem == 0 {
                    // TODO: Set errno to EOVERFLOW.
                    return -1;
                }
                if !print_bytes(&[c]) {
                    return -1;
                }
                written += 1;
            }
            b's' => {
                i += 1;
                let str_ptr: *const u8 = unsafe { args.arg::<*const u8>() };
                let len = strlen(str_ptr) as i32;
                if maxrem < len {
                    // TODO: Set errno to EOVERFLOW.
                    return -1;
                }
                let s = unsafe { core::slice::from_raw_parts(str_ptr, len as usize) };
                if !print_bytes(s) {
                    return -1;
                }
                written += len;
            }
            _ => {
                // Unknown specifier — print the rest of the format string as-is
                let remaining = &fmt[format_begun_at..];
                let len = remaining.len() as i32;
                if maxrem < len {
                    // TODO: Set errno to EOVERFLOW.
                    return -1;
                }
                if !print_bytes(remaining) {
                    return -1;
                }
                written += len;
                i = fmt.len();
            }
        }
    }

    return written;
}

// ── Safe Rust-only API ──────────────────────────────────────────────

/// Print a byte slice to the terminal. Fully safe — no raw pointers, no null
/// terminators required.
pub fn kprint(s: &[u8]) -> bool {
    print_bytes(s)
}

/// Print a byte slice followed by a newline.
pub fn kprintln(s: &[u8]) -> bool {
    print_bytes(s) && print_bytes(b"\n")
}