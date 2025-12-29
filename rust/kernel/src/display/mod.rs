pub(crate) mod vga;

use core::cell::SyncUnsafeCell;
use vga::{Writer, Color};

pub static WRITER: SyncUnsafeCell<Writer> = SyncUnsafeCell::new(
    Writer::new(
        Color::LightGray,
        Color::Black,
        0xB8000 as *mut u16
    )
);
