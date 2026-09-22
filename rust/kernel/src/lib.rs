#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod display;
mod scheduler;
mod io;
mod mem;

#[allow(dead_code)]
use core::fmt::Write;
use core::arch::asm;
use core::ptr::addr_of_mut;

use crate::display::WRITER;
use crate::scheduler::SCHEDULER;
//mod vga_buffer;

//use vga_buffer::{Color, Writer};

// Allocate stacks for tasks (in .bss section)
#[unsafe(link_section = ".bss")]
static mut TASK1_STACK: [u8; 4096] = [0; 4096];

#[unsafe(link_section = ".bss")]
static mut TASK2_STACK: [u8; 4096] = [0; 4096];

#[unsafe(link_section = ".bss")]
static mut TASK3_STACK: [u8; 4096] = [0; 4096];

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let writer = unsafe { &mut *WRITER.get() };
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
    let writer = unsafe {&mut *WRITER.get()};

    writer.write("[x] Vga Buffer initialized");
    writer.new_line();

    // Add tasks to scheduler
    let scheduler = unsafe { &mut *SCHEDULER.get() };

    let task1_base = addr_of_mut!(TASK1_STACK) as u64;
    let task2_base = addr_of_mut!(TASK2_STACK) as u64;
    let task3_base = addr_of_mut!(TASK3_STACK) as u64;
    
    scheduler.add_task(task1, task1_base, 4096);
    scheduler.add_task(task2, task2_base, 4096);
    scheduler.add_task(task3, task3_base, 4096);

    writer.write("[x] tasks added to scheduler");
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

    scheduler.kernel_dispatcher()
}

fn task1() -> ! {
    let mut counter = 0u64;
    loop {
        // Print to second line
        unsafe {
            let vga = (0xb8000) as *mut u8;
            let msg = b"Task 1 running";
            for (i, &byte) in msg.iter().enumerate() {
                *vga.offset((i * 2) as isize) = byte;
                *vga.offset((i * 2 + 1) as isize) = 0x0A;  
            }
            
            // Print counter
            *vga.offset(40) = b'0' + (counter % 10) as u8;
            counter += 1;
        }
        
        //for _ in 0..1000000 { unsafe { asm!("nop"); } }
    }
}

fn task2() -> ! {
    let mut counter = 0u64;

    loop {
        if counter % 100 == 0 {
        unsafe {
            let vga = (0xb8000 + 160) as *mut u8;
            let msg = b"Loop count: ";

            for (i, &byte) in msg.iter().enumerate() {
                *vga.offset((i * 2) as isize) = byte;
                *vga.offset((i * 2 + 1) as isize) = 0x0C;
            }

            // Convert `counter` into digits and print left-to-right
            let mut temp = counter;
            for digit_idx in (0..10).rev() {
                let digit = (temp % 10) as u8;
                temp /= 10;

                // Offset past "Loop count: " (12 chars)
                let offset = (12 + digit_idx) * 2;
                *vga.offset(offset as isize) = b'0' + digit;
                *vga.offset((offset + 1) as isize) = 0x0F; // White
            }
        }
        }

        counter = counter.wrapping_add(1);

        // Adjust or remove this loop to observe the change in count speed:
        for _ in 0..100_000 {
            unsafe { asm!("nop"); }
        }
    }
}

fn task3() -> ! {
    let mut counter = 0u64;
    loop {
        // Print to third line
        unsafe {
            let vga = (0xb8000 + 320) as *mut u8;  // Third line
            let msg = b"Task 3 running";
            for (i, &byte) in msg.iter().enumerate() {
                *vga.offset((i * 2) as isize) = byte;
                *vga.offset((i * 2 + 1) as isize) = 0x0E;  // Yellow
            }
            
            *vga.offset(40) = b'0' + (counter % 10) as u8;
            counter += 1;
        }
        
        for _ in 0..1000000 { unsafe { asm!("nop"); } }
    }
}
