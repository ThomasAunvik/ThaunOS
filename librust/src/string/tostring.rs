//! Integer-to-string conversion helpers for `no_std` environments.
//!
//! Every function writes into a caller-supplied `&mut [u8]` buffer and returns
//! the slice of bytes that contain the formatted number.  No heap allocation is
//! required.

/// Maximum number of digits for a 64-bit value (20 decimal digits + sign).
const MAX_I64_DIGITS: usize = 20;

// ── Core helpers (private) ──────────────────────────────────────────

/// Write the unsigned decimal representation of `n` into `buf` and return the
/// sub-slice that was written.  Panics (in debug) if `buf` is too small.
fn fmt_u64(n: u64, buf: &mut [u8]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }

    // Write digits in reverse order into a small stack buffer …
    let mut tmp = [0u8; MAX_I64_DIGITS];
    let mut len = 0usize;
    let mut val = n;
    while val > 0 {
        tmp[len] = b'0' + (val % 10) as u8;
        val /= 10;
        len += 1;
    }

    // … then copy them into `buf` in the correct (forward) order.
    for i in 0..len {
        buf[i] = tmp[len - 1 - i];
    }
    &buf[..len]
}

/// Write the signed decimal representation of `n` into `buf` and return the
/// sub-slice that was written.
fn fmt_i64(n: i64, buf: &mut [u8]) -> &[u8] {
    if n >= 0 {
        return fmt_u64(n as u64, buf);
    }
    buf[0] = b'-';
    // Handle i64::MIN safely: wrapping negate then treat as u64.
    let abs = (n.wrapping_neg()) as u64;
    let rest = fmt_u64(abs, &mut buf[1..]);
    let total = 1 + rest.len();
    &buf[..total]
}

// ── Public API ──────────────────────────────────────────────────────

/// Convert an `i8` to its decimal string representation.
///
/// ```ignore
/// let mut buf = [0u8; 8];
/// let s = i8_to_str(-42, &mut buf);   // s == b"-42"
/// ```
pub fn i8_to_str(n: i8, buf: &mut [u8]) -> &[u8] {
    fmt_i64(n as i64, buf)
}

/// Convert a `u8` to its decimal string representation.
pub fn u8_to_str(n: u8, buf: &mut [u8]) -> &[u8] {
    fmt_u64(n as u64, buf)
}

/// Convert an `i16` to its decimal string representation.
pub fn i16_to_str(n: i16, buf: &mut [u8]) -> &[u8] {
    fmt_i64(n as i64, buf)
}

/// Convert a `u16` to its decimal string representation.
pub fn u16_to_str(n: u16, buf: &mut [u8]) -> &[u8] {
    fmt_u64(n as u64, buf)
}

/// Convert an `i32` to its decimal string representation.
pub fn i32_to_str(n: i32, buf: &mut [u8]) -> &[u8] {
    fmt_i64(n as i64, buf)
}

/// Convert a `u32` to its decimal string representation.
pub fn u32_to_str(n: u32, buf: &mut [u8]) -> &[u8] {
    fmt_u64(n as u64, buf)
}

/// Convert an `i64` to its decimal string representation.
pub fn i64_to_str(n: i64, buf: &mut [u8]) -> &[u8] {
    fmt_i64(n, buf)
}

/// Convert a `u64` to its decimal string representation.
pub fn u64_to_str(n: u64, buf: &mut [u8]) -> &[u8] {
    fmt_u64(n, buf)
}

/// Convert a `usize` to its decimal string representation.
pub fn usize_to_str(n: usize, buf: &mut [u8]) -> &[u8] {
    fmt_u64(n as u64, buf)
}

/// Convert an `isize` to its decimal string representation.
pub fn isize_to_str(n: isize, buf: &mut [u8]) -> &[u8] {
    fmt_i64(n as i64, buf)
}

// ── Hex helpers ─────────────────────────────────────────────────────

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Write `n` as a lowercase hexadecimal string (no `0x` prefix) into `buf`.
pub fn u64_to_hex_str(n: u64, buf: &mut [u8]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }

    let mut tmp = [0u8; 16]; // 64 / 4 = 16 hex digits max
    let mut len = 0usize;
    let mut val = n;
    while val > 0 {
        tmp[len] = HEX_DIGITS[(val & 0xF) as usize];
        val >>= 4;
        len += 1;
    }

    for i in 0..len {
        buf[i] = tmp[len - 1 - i];
    }
    &buf[..len]
}

/// Write `n` as a lowercase hexadecimal string (no `0x` prefix) into `buf`.
pub fn usize_to_hex_str(n: usize, buf: &mut [u8]) -> &[u8] {
    u64_to_hex_str(n as u64, buf)
}
