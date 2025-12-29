use core::fmt;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;


#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Color {
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
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    ascii_code: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],  
}

impl Buffer {
    fn write(&mut self, row: usize, col: usize, char: ScreenChar) {
        unsafe { 
            let ptr = &mut self.chars[row][col] as *mut ScreenChar;
            core::ptr::write_volatile(ptr, char)
        };
    }

    fn read(&self, row: usize, col: usize) -> ScreenChar {
        unsafe { core::ptr::read_volatile(& self.chars[row][col]) }
    }
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: *mut Buffer
}

// Implementation since Writer is static and contains a pointer
unsafe impl Sync for Writer {}

impl Writer {
    
    pub const fn new(foreground: Color, background: Color, buffer_ptr: *mut u16) -> Writer {
        Writer {
            column_position: 0,
            color_code: ColorCode::new(foreground, background),
            buffer: buffer_ptr as *mut Buffer,
        }
    }
    
    pub fn write(&mut self, str: &str) {
        for ch in str.bytes() {
            self.write_byte(ch);
        }
    }

    #[inline]
    pub fn write_byte(&mut self, chr: u8) {
        match chr {
            b'\n' => {
                self.new_line();
            }
            byte => {
                self.send_bytes(VGA_HEIGHT - 1, self.column_position, byte);
                self.column_position += 1;

                if self.column_position >= VGA_WIDTH {
                    self.new_line();
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn clean(&mut self) {
        for i in 0..VGA_HEIGHT {
            for j in 0..VGA_WIDTH {
                self.send_bytes(i, j, b' ');       
            }
        }
    }

    fn send_bytes(&mut self, row: usize, col: usize, byte: u8) {
        let bytes = ScreenChar {
            color_code: self.color_code,
            ascii_code: byte
        };
        unsafe {
            (&mut *self.buffer).write(row, col, bytes);
        }
    }

    pub fn delete_last_char(&mut self) {
        let value = ScreenChar {
            color_code: self.color_code,
            ascii_code: b' '
        };
        let buffer = unsafe { &mut *self.buffer };

        if self.column_position == 0 {
            for i in (1..VGA_HEIGHT).rev() {
                for j in 0..VGA_WIDTH {
                    buffer.write(i, j, buffer.read(i - 1, j));
                }
            }

            // clear first line on the top
            for i in 0..VGA_WIDTH {
                buffer.write(0, i, value);
            }

            let mut i = VGA_WIDTH - 1;

            // position the cursor on the previous line
            while buffer.read(VGA_HEIGHT - 1, i).ascii_code == b' '
                && i != 0
            {
                i -= 1;
            }

            self.column_position = i + 1;

        } else {
            self.column_position -= 1;
            buffer.write(VGA_HEIGHT - 1, self.column_position, value);
        }
    }

    pub fn new_line(&mut self) {
        self.column_position = 0;
        let buffer = unsafe { &mut *self.buffer };
        
        for i in 0..(VGA_HEIGHT - 1) {
            for j in 0..VGA_WIDTH {
                let value = buffer.read(i + 1, j);
                buffer.write(i, j, value);
            }
        }

        let value = ScreenChar {
            color_code: self.color_code,
            ascii_code: b' '
        };
        for i in 0..VGA_WIDTH {
            buffer.write(VGA_HEIGHT - 1, i, value);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}
