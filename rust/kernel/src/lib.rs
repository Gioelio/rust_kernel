#![no_std]
#![no_main]

mod interrupts;
mod display;

use core::fmt::Write;
use core::arch::asm;

use crate::display::{init_writer, writer};
//mod vga_buffer;

//use vga_buffer::{Color, Writer};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let writer = unsafe { writer() };
    let _ = writeln!(writer, "\nPANIC: {}", info);
    halt()
}

fn halt() -> ! {
    unsafe {
        asm!("cli"); // Disable interrupts
        loop {
            asm!("hlt");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn start64() -> ! {
    // Init Writer Vga
    unsafe {
        init_writer();
    }

    let writer = unsafe { writer() };
    writer.write("[x] Vga Buffer initialized");
    writer.new_line();

    // Initialize interrupts
    interrupts::init_pic();
    interrupts::init_idt();

    // Enable interrupts
    unsafe {
        core::arch::asm!("sti"); // Set interrupt flag
    }

    writer.write("[x] Interrupts ready");
    writer.new_line();

    writer.write("----- System ready to be used ------");
    writer.new_line();
    writer.new_line();

    
    //panic!("test");
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

