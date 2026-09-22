mod keyboard;

use core::arch::asm;
use crate::{display::{WRITER, vga::Writer}, interrupts::keyboard::{Action, KeyType}, io, scheduler::SCHEDULER};
use keyboard::{Keyboard, KeyState};
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use io::{outb, inb};

pub static mut TIMER_TICKS: u64 = 0;
pub const SCHEDULE_INTERVAL: u64 = 100;

pub static mut IS_TIME_TO_SCHEDULE: bool = false;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,        // Lower 16 bits of handler address
    selector: u16,          // Code segment selector
    ist: u8,                // Interrupt Stack Table
    flags: u8,              // Type and attributes
    offset_mid: u16,        // Middle 16 bits of handler address
    offset_high: u32,       // Upper 32 bits of handler address
    reserved: u32,          // Must be zero
}

impl IdtEntry {
    pub const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64, selector: u16, flags: u8) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.selector = selector;
        self.ist = 0;
        self.flags = flags;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
pub struct IdtPointer {
    limit: u16,
    base: u64
}

// IDT with 256 entries
pub static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];

// TODO: move in periferal crate or module
pub static mut KEYBOARD: Keyboard = Keyboard::new();

/// Initialize PIC (Programmable Interrupt Controllers)
///
/// Initialize both Master and Slave PIC for 8259 PIC on x86 hw.
/// Remaps the dual 8259 PIC interrupt vectors from default (0x08-0x0F) to 0x10-0x1F (32-47).
///
/// By default, IRQ vectors overlap with CPU exceptions (0-31), misinterpreting false Page/Double
/// Faults as hw interrupts trigger. This 4-step ICW sequence offsets Master IRQs (0-7) to vectors
/// 32-39 and Slave IRQs (8-15) to vectors 40-47 in 8086 mode, then unmasks all hw interrupts lines.
pub fn init_pic() {
    unsafe {
        // ICW1: Initialize PIC (cascade mode)
        outb(0x20, 0x11);  // Master PIC
        outb(0xA0, 0x11);  // Slave PIC
        
        // ICW2: Remap IRQs
        // Master PIC: IRQ 0-7 → interrupts 32-39
        outb(0x21, 32);
        // Slave PIC: IRQ 8-15 → interrupts 40-47
        outb(0xA1, 40);
        
        // ICW3: Tell master about slave at IRQ2
        outb(0x21, 0x04);
        // Tell slave its cascade identity
        outb(0xA1, 0x02);
        
        // ICW4: 8086 mode
        outb(0x21, 0x01);
        outb(0xA1, 0x01);
        
        // Unmask all IRQs (enable them)
        outb(0x21, 0x00);
        outb(0xA1, 0x00);
    }
}

/// Interrupt Descriptor Table
///
/// Connect the interrupts to the instruction that should be executed when fired.
pub fn init_idt() {
    unsafe {
        // Set up exception handlers (interrupts 0-31)
        // Flags 0x8E: Presetn, DPL (00 = Kernel level), Storage segment, Gate type (64-bit
        // interrupt gate)
        //IDT[0].set_handler(divide_by_zero_handler as *const () as u64, 0x08, 0x8E);
        //IDT[13].set_handler(general_protection_fault_handler as *const () as u64, 0x08, 0x8E);
        IDT[14].set_handler(page_fault_handler as *const () as u64, 0x08, 0x8E);
        
        // Set up IRQ handlers (interrupts 32-47)
        IDT[32].set_handler(timer_handler as *const () as u64, 0x08, 0x8E);
        IDT[33].set_handler(keyboard_handler as *const () as u64, 0x08, 0x8E);
        
        // Load IDT with LIDT
        let idt_ptr = IdtPointer {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: &raw const IDT as *const _ as u64,
        };

        asm!("lidt [{}]", in(reg) &idt_ptr, options(readonly, nostack, preserves_flags));
    }
}

#[repr(C)]
pub struct InterruptFrame {
    r15: u64, r14: u64, r13: u64, r12: u64,
    r11: u64, r10: u64, r9: u64, r8: u64,
    rbp: u64, rdi: u64, rsi: u64, rdx: u64,
    rcx: u64, rbx: u64, rax: u64,
    interrupt_number: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

pub extern "x86-interrupt" fn timer_handler(_frame: InterruptStackFrame) {
    unsafe {
        // Increase timer ticks
        TIMER_TICKS = TIMER_TICKS.wrapping_add(1);

        // send EOI to Master PIC
        outb(0x20, 0x20);

        // Check end interval and re-schedule next tasks
        if TIMER_TICKS.is_multiple_of(SCHEDULE_INTERVAL) {
            (&mut *SCHEDULER.get()).return_to_kernel();
        }

    }
}

pub extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    // store inputs in temporary buffer
    #[allow(static_mut_refs)]
    unsafe { KEYBOARD.interrupt_handler(); };

    io::send_eoi();
   
    #[allow(static_mut_refs)]
    unsafe { KEYBOARD.post_interrupt(); };
}

pub extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64
) {
    panic!("Page Fault! Frame: {:#?}, Error Code: {:#?}", frame, error_code);
}
