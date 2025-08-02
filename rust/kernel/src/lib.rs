#![no_std]
#![no_main]

use core::{fmt::Write};


//extern crate libc;

mod display;
use core::arch::asm;
use display::vga::{Writer, Color};
//mod vga_buffer;

//use vga_buffer::{Color, Writer};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Try to print panic info to VGA buffer
    let vga_buffer = 0xB8000 as *mut u16;
    let mut writer = Writer::new(Color::Black, Color::LightRed, vga_buffer);
    let _ = writeln!(writer, "PANIC: {}", info);
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

#[no_mangle]
pub extern "C" fn start64() -> ! {
    let vga_buffer = 0xB8000 as *mut u16;
    let mut writer = Writer::new(Color::Black, Color::Green, vga_buffer);
    writer.clean();
    //writer.clean();
    //writer.write("x");
    //writer.clean();
    //writer.clean();
    //writer.write("x");
    //writer.new_line();
    //writer.new_line();
    writer.write("Hello\n");
    //writer.flush();
    writer.write("Hello\n world!\n");
    writer.write("ciaociaociao");
    writer.new_line();
    //writer.write("ciao!\n");
    //writer.write("ciao");
    //writer.flush();
    //display::vga::terminal_init();
    //display::vga::terminal_writestring("Hello world!");
    panic!("test");
    halt()
}

