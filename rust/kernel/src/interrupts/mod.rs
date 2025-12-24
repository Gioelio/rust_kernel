mod keyboard;

use core::arch::asm;
use crate::{display::writer, interrupts::keyboard::{Action, KeyType}};
use keyboard::{Keyboard, KeyState};


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

pub fn init_idt() {
    unsafe {
        // Set up exception handlers (interrupts 0-31)
        // Flags 0x8E: Presetn, DPL (00 = Kernel level), Storage segment, Gate type (64-bit
        // interrupt gate)
        IDT[0].set_handler(divide_by_zero_handler as u64, 0x08, 0x8E);
        IDT[13].set_handler(general_protection_fault_handler as u64, 0x08, 0x8E);
        IDT[14].set_handler(page_fault_handler as u64, 0x08, 0x8E);
        
        // Set up IRQ handlers (interrupts 32-47)
        IDT[32].set_handler(timer_handler as u64, 0x08, 0x8E);
        IDT[33].set_handler(keyboard_handler as u64, 0x08, 0x8E);
        
        // Load IDT
        let idt_ptr = IdtPointer {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: &raw const IDT as *const _ as u64,
        };

        asm!("lidt [{}]", in(reg) &idt_ptr, options(readonly, nostack, preserves_flags));
    }
}

// External assembly handlers
unsafe extern "C" {
    fn divide_by_zero_handler();
    fn general_protection_fault_handler();
    fn page_fault_handler();
    fn timer_handler();
    fn keyboard_handler();
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

#[unsafe(no_mangle)]
pub extern "C" fn rust_exception_handler(frame: &InterruptFrame) {
    // TODO: ADD VGA printing here to show errors
    match frame.interrupt_number {
        0 => {
            // Divide by zero
            // Print error message
            loop {}  // Halt
        }
        13 => {
            // General protection fault
            // Print error and registers
            loop {}
        }
        14 => {
            // Page fault
            // You can read CR2 register to get fault address
            loop {}
        }
        _ => {
            // Unknown exception
            loop {}
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_irq_handler(frame: &InterruptFrame) {
    let irq = frame.interrupt_number - 32;
    
    match irq {
        0 => {
            // Timer interrupt
            // You might increment a tick counter here
        }
        1 => {
            let writer = unsafe { writer() };
            // Keyboard interrupt
            // Read scancode from port 0x60
            let scancode = unsafe { inb(0x60) };
            #[allow(static_mut_refs)]
            let key_info = unsafe { KEYBOARD.scan(scancode) };
            
            if key_info.state == KeyState::Pressed {
                if let Some(chr) = key_info.key.print() { 
                    writer.write_byte(chr);
                }
                else if let KeyType::Action(action) = key_info.key {
                    match action {
                        Action::Delete => {
                            writer.delete_last_char();
                        },
                        _ => {}
                    }
                }
            }
            // Process keyboard input
        }
        _ => {}
    }
    
    // Send End of Interrupt (EOI) signal to PIC
    unsafe {
        if irq >= 8 {
            outb(0xA0, 0x20);  // EOI to slave PIC
        }
        outb(0x20, 0x20);      // EOI to master PIC
    }
}

// Port I/O helper functions
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

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
