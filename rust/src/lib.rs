#![no_std]


//extern crate libc;

mod display;
mod libc;
use core::arch::asm;


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {} // Halt the program
}

fn halt() -> ! {
    unsafe {
        asm!("hlt");
        loop {}
    }
}

#[no_mangle]
pub extern "C" fn start64() -> ! {
    display::vga::terminal_init();
    display::vga::terminal_writestring("Hello world!");
    halt()
}
