use core::ffi::c_void;
use core::ptr::{copy_nonoverlapping, write_bytes};

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut c_void  {
    // TODO: implement paging to handle memory allocation
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut c_void) {
    // TOOD: implement memory deallocation
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    copy_nonoverlapping(src, dest, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    write_bytes(s, c as u8, n);
    s
}
