pub(crate) mod vga;
use core::fmt::{self, Write};

use core::cell::SyncUnsafeCell;
use vga::{Color, Writer};

pub static WRITER: SyncUnsafeCell<Writer> = SyncUnsafeCell::new(Writer::new(
    Color::LightGray,
    Color::Black,
    0xB8000 as *mut u16,
));

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::display::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        (*WRITER.get()).write_fmt(args).unwrap();
    }
}
