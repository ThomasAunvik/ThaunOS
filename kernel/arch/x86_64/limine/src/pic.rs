//! 8259 PIC (Programmable Interrupt Controller) driver.
//!
//! Remaps IRQ 0-15 to IDT vectors 0x20-0x2F so they don't collide with
//! CPU exceptions (vectors 0-31).

use crate::port::{inb, outb, io_wait};

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

/// Remap the PIC so that IRQ 0-7  → vectors 0x20-0x27
///                     and IRQ 8-15 → vectors 0x28-0x2F.
///
/// After remapping, all IRQs are masked except the ones we explicitly enable.
pub unsafe fn init() {
    unsafe {
        // Save existing masks
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // ICW1: start init sequence (cascade mode, ICW4 needed)
        outb(PIC1_CMD, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_CMD, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // ICW2: vector offsets
        outb(PIC1_DATA, 0x20); // IRQ 0-7  → 0x20-0x27
        io_wait();
        outb(PIC2_DATA, 0x28); // IRQ 8-15 → 0x28-0x2F
        io_wait();

        // ICW3: cascading
        outb(PIC1_DATA, 0x04); // slave PIC on IRQ2
        io_wait();
        outb(PIC2_DATA, 0x02); // slave identity = 2
        io_wait();

        // ICW4: 8086 mode
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Restore saved masks
        outb(PIC1_DATA, mask1);
        io_wait();
        outb(PIC2_DATA, mask2);
        io_wait();
    }
}

/// Unmask (enable) a specific IRQ line (0-15).
pub unsafe fn unmask_irq(irq: u8) {
    unsafe {
        if irq < 8 {
            let mask = inb(PIC1_DATA) & !(1 << irq);
            outb(PIC1_DATA, mask);
        } else {
            let mask = inb(PIC2_DATA) & !(1 << (irq - 8));
            outb(PIC2_DATA, mask);
            // Also unmask IRQ2 (cascade) on the master PIC
            let master = inb(PIC1_DATA) & !(1 << 2);
            outb(PIC1_DATA, master);
        }
    }
}

/// Mask (disable) all IRQs.
pub unsafe fn mask_all() {
    unsafe {
        outb(PIC1_DATA, 0xFF);
        io_wait();
        outb(PIC2_DATA, 0xFF);
        io_wait();
    }
}

/// Send End-Of-Interrupt to the appropriate PIC(s).
pub unsafe fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_CMD, 0x20);
        }
        outb(PIC1_CMD, 0x20);
    }
}
