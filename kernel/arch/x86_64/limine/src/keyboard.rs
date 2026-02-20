//! PS/2 keyboard driver (IRQ1, scancode set 1).
//!
//! Provides a ring buffer that the IRQ handler fills with ASCII characters.
//! The kernel can poll with [`try_read_char`] or spin with [`read_char`].

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::pic;
use crate::port::inb;

// ── Ring buffer for received characters ─────────────────────────────

const BUF_SIZE: usize = 256;

/// Ring buffer storage.  Written only from the IRQ handler (single-producer).
static mut KEY_BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];
/// Write index (only modified by the IRQ handler).
static WRITE_IDX: AtomicUsize = AtomicUsize::new(0);
/// Read index (only modified by the consumer).
static READ_IDX: AtomicUsize = AtomicUsize::new(0);

// ── Shift / modifier tracking ───────────────────────────────────────

static SHIFT_HELD: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK: AtomicBool = AtomicBool::new(false);

// ── Scancode set 1 → ASCII tables (US QWERTY) ──────────────────────

/// Lower-case / unshifted mapping for scancodes 0x00 – 0x39.
static SCANCODE_TO_ASCII: [u8; 58] = [
    0,   27,  b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', // 0x00-0x09
    b'9', b'0', b'-', b'=', 8,   b'\t',                         // 0x0A-0x0F
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', // 0x10-0x19
    b'[', b']', b'\n', 0,                                        // 0x1A-0x1D (0x1D = L-Ctrl)
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l',      // 0x1E-0x26
    b';', b'\'', b'`', 0,    b'\\',                              // 0x27-0x2B (0x2A = L-Shift)
    b'z', b'x', b'c', b'v', b'b', b'n', b'm',                   // 0x2C-0x32
    b',', b'.', b'/', 0,                                          // 0x33-0x36 (0x36 = R-Shift)
    b'*', 0, b' ',                                                // 0x37-0x39 (0x38 = L-Alt, 0x39 = Space)
];

/// Shifted mapping for the same range.
static SCANCODE_TO_ASCII_SHIFT: [u8; 58] = [
    0,   27,  b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*',   // 0x00-0x09
    b'(', b')', b'_', b'+', 8,   b'\t',                           // 0x0A-0x0F
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', // 0x10-0x19
    b'{', b'}', b'\n', 0,                                          // 0x1A-0x1D
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L',       // 0x1E-0x26
    b':', b'"', b'~', 0,    b'|',                                  // 0x27-0x2B
    b'Z', b'X', b'C', b'V', b'B', b'N', b'M',                    // 0x2C-0x32
    b'<', b'>', b'?', 0,                                           // 0x33-0x36
    b'*', 0, b' ',                                                 // 0x37-0x39
];

// ── IRQ handler (called from the IDT stub) ──────────────────────────

/// Called by the assembly IRQ stub.  Reads the scancode, translates it,
/// and pushes printable characters into the ring buffer.
#[unsafe(no_mangle)]
pub extern "C" fn keyboard_irq_handler() {
    // SAFETY: we are inside an interrupt with IF=0, so no preemption.
    unsafe {
        let scancode = inb(0x60);

        // Key release (bit 7 set)?
        if scancode & 0x80 != 0 {
            let released = scancode & 0x7F;
            // Left Shift = 0x2A, Right Shift = 0x36
            if released == 0x2A || released == 0x36 {
                SHIFT_HELD.store(false, Ordering::Relaxed);
            }
        } else {
            // Key press
            match scancode {
                0x2A | 0x36 => {
                    SHIFT_HELD.store(true, Ordering::Relaxed);
                }
                0x3A => {
                    // Caps Lock toggle
                    let prev = CAPS_LOCK.load(Ordering::Relaxed);
                    CAPS_LOCK.store(!prev, Ordering::Relaxed);
                }
                _ => {
                    if (scancode as usize) < SCANCODE_TO_ASCII.len() {
                        let shifted = SHIFT_HELD.load(Ordering::Relaxed);
                        let caps = CAPS_LOCK.load(Ordering::Relaxed);

                        let mut ch = if shifted {
                            SCANCODE_TO_ASCII_SHIFT[scancode as usize]
                        } else {
                            SCANCODE_TO_ASCII[scancode as usize]
                        };

                        // Caps Lock only affects letters
                        if caps && ch.is_ascii_alphabetic() {
                            ch = if shifted {
                                ch.to_ascii_lowercase()
                            } else {
                                ch.to_ascii_uppercase()
                            };
                        }

                        if ch != 0 {
                            let w = WRITE_IDX.load(Ordering::Relaxed);
                            let next = (w + 1) % BUF_SIZE;
                            // Drop the character if the buffer is full.
                            if next != READ_IDX.load(Ordering::Relaxed) {
                                KEY_BUF[w] = ch;
                                WRITE_IDX.store(next, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        }

        pic::send_eoi(1);
    }
}

// ── Public API ──────────────────────────────────────────────────────

/// Try to read one character from the keyboard buffer.
/// Returns `None` immediately if the buffer is empty.
pub fn try_read_char() -> Option<u8> {
    let r = READ_IDX.load(Ordering::Acquire);
    let w = WRITE_IDX.load(Ordering::Acquire);
    if r == w {
        return None;
    }
    // SAFETY: only one consumer; IRQ handler only advances WRITE_IDX.
    let ch = unsafe { KEY_BUF[r] };
    READ_IDX.store((r + 1) % BUF_SIZE, Ordering::Release);
    Some(ch)
}

/// Block until a character is available, then return it.
pub fn read_char() -> u8 {
    loop {
        if let Some(ch) = try_read_char() {
            return ch;
        }
        core::hint::spin_loop();
    }
}

/// Read characters until a newline (`\n`) is received, filling `buf`.
/// Returns the number of bytes written (excluding the newline).
/// The newline itself is *not* stored.
pub fn read_line(buf: &mut [u8]) -> usize {
    let mut i = 0;
    loop {
        let ch = read_char();
        if ch == b'\n' || ch == b'\r' {
            return i;
        }
        if ch == 8 {
            // Backspace
            if i > 0 {
                i -= 1;
            }
            continue;
        }
        if i < buf.len() {
            buf[i] = ch;
            i += 1;
        }
    }
}
