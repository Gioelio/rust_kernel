#![feature(asm, core_intrinsics, lang_items)]

#![no_std]

use core::intrinsics;
use core::arch::asm;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {} // Halt the program
}

fn halt() -> ! {
    unsafe {
        asm!("hlt");
        intrinsics::unreachable();
    }
}

#[no_mangle]
pub extern "C" fn start64() -> ! {
    halt()
}


#[lang = "eh_personality"]
fn eh_personality() { }

// memset
#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u8;
    for i in 0..n {
        *dest.add(i) = c;
    }
    dest
}

// memcpy
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

// bcmp (compare two memory regions)
#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        if *s1.add(i) != *s2.add(i) {
            return 1; // Not equal
        }
    }
    0 // Equal
}
