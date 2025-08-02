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
struct Entry(u16);

impl Entry {
    fn new(character: u8, foreground: Color, background: Color) -> Entry {
        let color = foreground as u8 | ((background as u8) << 4);
        let bytes = character as u16 | ((color as u16) << 8);

        Entry(bytes)
    }
}

#[repr(transparent)]
struct Buffer {
    chars: [[u16; VGA_WIDTH]; VGA_HEIGHT],  
}

impl Buffer {
    fn write(&mut self, row: usize, col: usize, value: u16) {
        unsafe { core::ptr::write_volatile(&mut self.chars[row][col], value) };
    }

    fn read(&self, row: usize, col: usize) -> u16 {
        unsafe { core::ptr::read_volatile(& self.chars[row][col]) }
    }
}

pub struct Writer {
    column_position: usize,
    foreground: Color,
    background: Color,
    buffer: &'static mut Buffer
}

impl Writer {
    
    pub fn new(foreground: Color, background: Color, buffer_ptr: *mut u16) -> Writer {
        Writer {
            column_position: 0,
            foreground,
            background,
            buffer: unsafe { &mut *(buffer_ptr as *mut Buffer) },
        }
    }
    
    pub fn write(&mut self, str: &str) {
        for ch in str.bytes() {
            match ch {
                b'\n' => {
                    self.new_line();
                }
                byte => {
                    if self.column_position >= VGA_WIDTH {
                        self.new_line();
                    }

                    self.send_bytes(VGA_HEIGHT - 1, self.column_position, byte);
                    self.column_position += 1;
                }
            }
        }
    }

    pub fn clean(&mut self) {
        for i in 0..VGA_HEIGHT {
            for j in 0..VGA_WIDTH {
                self.send_bytes(i, j, b' ');       
            }
        }
    }

    fn send_bytes(&mut self, row: usize, col: usize, byte: u8) {
        let bytes = Entry::new(byte, self.foreground, self.background).0;
        self.buffer.write(row, col, bytes);
    }

    pub fn new_line(&mut self) {
        self.column_position = 0;
        
        for i in 0..(VGA_HEIGHT - 1) {
            for j in 0..VGA_WIDTH {
                let value = self.buffer.read(i + 1, j);
                self.buffer.write(i, j, value);
            }
        }

        let value = Entry::new(b' ', self.foreground, self.background);
        for i in 0..VGA_WIDTH {
            self.buffer.write(VGA_HEIGHT - 1, i, value.0);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}
