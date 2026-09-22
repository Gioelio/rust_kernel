use core::arch::asm;

#[inline]
pub fn read_keyboard() -> u8 {
    return unsafe { inb(0x60) }
}

#[inline]
/// Send End Of Interrupt (EOI) to Master PIC
pub fn send_eoi() {
   unsafe { outb(0x20, 0x20); }
}

// Port I/O general helper functions

/// Output bytes to a port
#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    }
}

/// Read bytes from a port
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    }
    value
}
