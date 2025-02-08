
const VGA_WIDTH: isize = 80;
const VGA_HEIGHT: isize = 15;
const TERMINAL_BUFFER: *mut u16 = 0xB8000 as *mut u16; 


#[derive(PartialEq, Eq)]
#[repr(u8)]
enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    fg as u8 | ((bg as u8) << 4)
}

fn vga_entry(uc: char, color: u8) -> u16 {
    uc as u16 | ((color as u16) << 8)
}

fn terminal_putchar(idx: isize, ch: char) {
    let terminal_color = vga_entry_color(VgaColor::White, VgaColor::DarkGray);
    unsafe {
        let ptr = TERMINAL_BUFFER.offset(idx); 
        *ptr = vga_entry(ch, terminal_color);
    }

}

pub fn terminal_init() {

    for y in 0..VGA_HEIGHT {
        for x in 0..VGA_WIDTH {
            //TODO: understand why this skip is necessary to avoid crash
            if x == 0 && y == 0 {
                continue;
            }
            let index: isize = y * VGA_WIDTH + x;
            terminal_putchar(index, ' ');
        }
    }

    terminal_putchar(0_isize, ' ');
}

pub fn terminal_writestring(str: &str) {
    let mut idx = 0_isize;
    for ch in str.bytes() {
        terminal_putchar(idx, ch as char);
        idx += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bit_shifting() {
        assert_eq!(vga_entry_color(VgaColor::Blue, VgaColor::Blue), 0b00010001); 
    }

}
