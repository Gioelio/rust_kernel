#![no_std]
#![no_main]

mod interrupts;
mod display;
mod scheduler;

#[allow(dead_code)]
use core::fmt::Write;
use core::arch::asm;
use core::ptr::addr_of_mut;

use crate::{display::{init_writer, writer}, scheduler::init_scheduler};
use crate::scheduler::scheduler_instance;
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

    unsafe {
        init_scheduler();
    }

    writer.write("[x] scheduler ready");
    writer.new_line();

    // Initialize interrupts
    interrupts::init_pic();
    interrupts::init_idt();

    // Add tasks to scheduler
    unsafe {
        let task1_base = addr_of_mut!(TASK1_STACK) as u64;
        let task2_base = addr_of_mut!(TASK2_STACK) as u64;
        let task3_base = addr_of_mut!(TASK3_STACK) as u64;
        
        scheduler_instance().add_task(task1, task1_base, 4096);
        scheduler_instance().add_task(task2, task2_base, 4096);
        scheduler_instance().add_task(task3, task3_base, 4096);
        
    }

    // Enable interrupts
    unsafe {
        core::arch::asm!("sti"); // Set interrupt flag
    }

    writer.write("[x] Interrupts ready");
    writer.new_line();

    writer.write("----- System ready to be used ------");
    writer.new_line();
    writer.new_line();

    unsafe {
        scheduler_instance().kernel_dispatcher();
    }

}

// Kernel idle task - handles deferred interrupts
fn kernel_idle_task() -> ! {
    let writer = unsafe { writer() };
    
    loop {
        // Handle keyboard interrupt

        // Pause CPU until next interrupt
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
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
        // Print to second line
        unsafe {
            let vga = (0xb8000 + 160) as *mut u8;  // 160 = 80 chars * 2 bytes
            let msg = b"Task 2 running";
            for (i, &byte) in msg.iter().enumerate() {
                *vga.offset((i * 2) as isize) = byte;
                *vga.offset((i * 2 + 1) as isize) = 0x0C;  // Red
            }
            
            // Print counter
            *vga.offset(40) = b'0' + (counter % 10) as u8;
            counter += 1;
        }
        
        for _ in 0..1000000 { unsafe { asm!("nop"); } }
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
