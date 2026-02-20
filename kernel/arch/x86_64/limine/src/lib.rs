#![no_std]
#![crate_name = "limine"]
#![allow(non_camel_case_types, non_upper_case_globals)]

mod bindings;
pub mod keyboard;
pub mod pic;
pub mod port;

pub use bindings::*;

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

// ── Default exception handler (assembly) ────────────────────────────

core::arch::global_asm!(
    "_default_exception_handler:",
    "cli",
    "2: hlt",
    "jmp 2b",
);

// IRQ1 (keyboard) stub: skips the red zone, saves all GP registers,
// calls the Rust handler, restores registers, then returns from interrupt.
core::arch::global_asm!(
    "_keyboard_irq_stub:",
    // Skip past the 128-byte red zone (x86_64 SysV ABI).
    "sub rsp, 128",
    // Save all general-purpose registers
    "push rax",
    "push rcx",
    "push rdx",
    "push rbx",
    "push rbp",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    // Call the Rust handler
    "call keyboard_irq_handler",
    // Restore all general-purpose registers
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rbp",
    "pop rbx",
    "pop rdx",
    "pop rcx",
    "pop rax",
    // Restore RSP past the red zone
    "add rsp, 128",
    "iretq",
);

// Dummy handler for IRQ vectors we don't care about (e.g. timer).
// Sends EOI to both PICs and returns — prevents the default halt handler
// from freezing the system when a spurious or unhandled IRQ fires.
core::arch::global_asm!(
    "_dummy_irq_stub:",
    "push rax",
    // Send EOI to slave PIC (in case IRQ >= 8)
    "mov al, 0x20",
    "out 0xA0, al",
    // Send EOI to master PIC
    "out 0x20, al",
    "pop rax",
    "iretq",
);

unsafe extern "C" {
    fn _default_exception_handler();
    fn _keyboard_irq_stub();
    fn _dummy_irq_stub();
}

// ── IDT types ───────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64, selector: u16) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.selector = selector;
        self.ist = 0;
        self.type_attr = 0x8E; // Present, DPL=0, 64-bit Interrupt Gate
        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

/// IDT storage, wrapped in UnsafeCell so we can initialise it once during boot.
struct IdtCell(UnsafeCell<[IdtEntry; 256]>);
unsafe impl Sync for IdtCell {}

static IDT: IdtCell = IdtCell(UnsafeCell::new([IdtEntry::empty(); 256]));

/// Populate every IDT slot with a default "halt" handler, install the
/// keyboard IRQ handler at vector 0x21, and load the IDT.
unsafe fn setup_idt() {
    let handler = _default_exception_handler as *const () as u64;

    // SAFETY: single-threaded init context; no other references to IDT exist.
    let idt = unsafe { &mut *IDT.0.get() };

    // Fill all 256 entries — CS selector 0x28 is the 64-bit code segment
    // set up by Limine's GDT.
    for entry in idt.iter_mut() {
        entry.set_handler(handler, 0x28);
    }

    // Install a dummy "send-EOI-and-return" handler for all 16 IRQ vectors
    // (0x20-0x2F) so that unhandled IRQs (like the PIT timer on IRQ0)
    // don't fall through to the default halt handler.
    let dummy = _dummy_irq_stub as *const () as u64;
    for vec in 0x20..=0x2F {
        idt[vec].set_handler(dummy, 0x28);
    }

    // Install the keyboard IRQ handler at vector 0x21 (IRQ1 after PIC remap).
    let kb_handler = _keyboard_irq_stub as *const () as u64;
    idt[0x21].set_handler(kb_handler, 0x28);

    let idt_ptr = IdtPtr {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: idt.as_ptr() as u64,
    };

    // SAFETY: idt_ptr points to a valid, fully-initialised IDT.
    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idt_ptr,
            options(nostack),
        );
    }
}

// ── Public framebuffer info ─────────────────────────────────────────

/// Runtime information about the Limine framebuffer, populated during `init()`.
pub struct FramebufferInfo {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
}

unsafe impl Send for FramebufferInfo {}
unsafe impl Sync for FramebufferInfo {}

/// Cell for the framebuffer info, writable once during init.
struct FbCell(UnsafeCell<MaybeUninit<FramebufferInfo>>);
unsafe impl Sync for FbCell {}

static FB_INFO: FbCell = FbCell(UnsafeCell::new(MaybeUninit::uninit()));
static FB_INIT: AtomicBool = AtomicBool::new(false);

/// Returns a reference to the framebuffer info if `init()` has been called.
pub fn framebuffer_info() -> Option<&'static FramebufferInfo> {
    if FB_INIT.load(Ordering::Acquire) {
        Some(unsafe { (*FB_INFO.0.get()).assume_init_ref() })
    } else {
        None
    }
}

// ── Helpers for linker-visible mutable statics ──────────────────────

/// A wrapper that makes a `T` visible to the linker as mutable (`static mut`
/// semantics) while keeping it safe to declare at file scope. The Limine
/// protocol requires several statics to be writable so the bootloader can
/// patch them before the kernel gains control.
#[repr(transparent)]
struct VolatileCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for VolatileCell<T> {}

impl<T> VolatileCell<T> {
    const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }
}

// ── Base revision ───────────────────────────────────────────────────

// Set the base revision to 4. This is the latest base revision described
// by the Limine boot protocol specification.
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".limine_requests")]
static limine_base_revision: VolatileCell<[u64; 3]> = VolatileCell::new([
    0xf9562b2d5c95a6c8,
    0x6a7b384944536bdc,
    4, // revision number
]);

// ── Framebuffer request ─────────────────────────────────────────────

// The Limine requests can be placed anywhere, but the compiler must not
// optimise them away, hence `#[used]`.
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".limine_requests")]
pub static limine_framebuffer_request: VolatileCell<limine_framebuffer_request> =
    VolatileCell::new(limine_framebuffer_request {
        id: [
            0xc7b1dd30df4c8b88,
            0x0a82e883a194f07b,
            0x9d5827dcd881dd75,
            0xa3148604f6fab11b,
        ],
        revision: 0,
        response: ptr::null_mut(),
    });

// ── Request section markers ─────────────────────────────────────────

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".limine_requests_start")]
static limine_requests_start_marker: VolatileCell<[u64; 4]> = VolatileCell::new([
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
]);

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".limine_requests_end")]
static limine_requests_end_marker: VolatileCell<[u64; 2]> = VolatileCell::new([
    0xadc0e0531bb10d03,
    0x9572709f31764c62,
]);


/// Halt and catch fire — loops forever.
fn hcf() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn init() {
    // SAFETY: These statics are written by the bootloader before we run.
    // We only read them here, in single-threaded init context.
    unsafe {
        // Set up the IDT first, so any subsequent exception is caught
        // instead of causing a triple fault.
        setup_idt();

        // Ensure the bootloader understands our base revision (see spec).
        // The bootloader zeroes element [2] to signal support.
        let base_rev = &*limine_base_revision.0.get();
        if base_rev[2] != 0 {
            hcf();
        }

        // Ensure we got a framebuffer.
        let request = &*limine_framebuffer_request.0.get();
        let response = request.response;
        if response.is_null() || (*response).framebuffer_count < 1 {
            hcf();
        }

        // Fetch the first framebuffer and store its info for other crates.
        let framebuffer = *(*response).framebuffers;

        (*FB_INFO.0.get()).write(FramebufferInfo {
            address: (*framebuffer).address as *mut u8,
            width: (*framebuffer).width,
            height: (*framebuffer).height,
            pitch: (*framebuffer).pitch,
            bpp: (*framebuffer).bpp,
        });
        FB_INIT.store(true, Ordering::Release);

        // Initialise the 8259 PIC: remap IRQs to vectors 0x20-0x2F,
        // mask everything, then unmask IRQ1 (keyboard).
        pic::mask_all();
        pic::init();
        pic::unmask_irq(1);

        // Enable hardware interrupts so the keyboard IRQ fires.
        core::arch::asm!("sti", options(nomem, nostack));
    }
}