pub(crate) mod vga;

use core::mem::MaybeUninit;
use vga::{Writer, Color};

static mut WRITER: MaybeUninit<Writer> = MaybeUninit::uninit();

// TODO: remove the allow and replace with MUTEX
#[allow(static_mut_refs)]
pub unsafe fn init_writer() {
    WRITER.write(Writer::new(
        Color::LightGray,
        Color::Black,
        0xB8000 as *mut u16,
    ));
}

#[inline]
// TODO: remove the allow and replace with MUTEX
#[allow(static_mut_refs)]
pub unsafe fn writer() -> &'static mut Writer {
    WRITER.assume_init_mut()
}
