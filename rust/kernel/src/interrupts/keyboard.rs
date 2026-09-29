use crate::mem::structures::RingBuffer;
use crate::{display::WRITER, io::read_keyboard};

/// Maximum amount of character stored between subsequent keyboard interrupts
const MAX_TEMP_CHARACTERS: usize = 128;

#[derive(PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(PartialEq)]
pub enum Action {
    Delete,
    Space,
    Enter,
    Shift,
}

#[derive(PartialEq)]
pub enum KeyType {
    Character(u8),
    Number(u8),
    Action(Action),
    Undefined(u8),
}

impl KeyType {
    pub fn print(&self) -> Option<u8> {
        match *self {
            Self::Character(c) => Some(c),
            Self::Number(n) => Some(n),
            _ => None,
        }
    }
}

#[derive(PartialEq)]
pub struct KeyPressed {
    pub key: KeyType,
    pub state: KeyState,
}

pub struct Keyboard {
    pub shift_enabled: bool,

    pub ring_buffer: RingBuffer<u8, MAX_TEMP_CHARACTERS>,
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard {
            shift_enabled: false,
            ring_buffer: RingBuffer::new(0),
        }
    }

    /// Store inputs in a temporary buffer to allow deferred usage
    pub fn interrupt_handler(&self) {
        let scancode = read_keyboard();

        // TODO: the error should Increase the priority of the consumer
        let _ = self.ring_buffer.push(scancode);
    }

    /// Use the temporary buffer to make use of the keyboard inputs
    pub fn post_interrupt(&mut self) {
        let writer = unsafe { &mut *WRITER.get() };

        while let Some(key_input) = self.ring_buffer.pop() {
            let key_info = self.scan(key_input);

            // TODO: eventually replace this with a function pointer
            if key_info.state == KeyState::Pressed {
                if let Some(chr) = key_info.key.print() {
                    writer.write_byte(chr);
                } else if let KeyType::Action(action) = key_info.key {
                    match action {
                        Action::Delete => {
                            writer.delete_last_char();
                        }
                        Action::Enter => {
                            writer.new_line();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn is_number(&self, ascii_code: u8) -> bool {
        matches!(ascii_code, 0x30..=0x39)
    }

    pub fn is_alphabet(ascii_code: u8) -> bool {
        matches!(ascii_code, 0x61..=0x7A | 0x41..=0x5A)
    }

    pub fn is_printable(&self, ascii_code: u8) -> bool {
        Keyboard::is_alphabet(ascii_code) || self.is_number(ascii_code)
    }

    pub fn scan(&mut self, mut scancode: u8) -> KeyPressed {
        let state = if scancode & 0x80 != 0 {
            scancode -= 0x80;
            KeyState::Released
        } else {
            KeyState::Pressed
        };

        let mut key = match scancode {
            0x0B => KeyType::Number(0x30),                      // 0
            0x02..=0x0A => KeyType::Number(scancode + 0x2F),    // 1-9
            0x1E => KeyType::Character(0x61),                   // A
            0x30 => KeyType::Character(0x62),                   // B
            0x2E => KeyType::Character(0x63),                   // C
            0x20 => KeyType::Character(0x64),                   // D
            0x12 => KeyType::Character(0x65),                   // E
            0x21..=0x23 => KeyType::Character(scancode + 0x45), // F-H
            0x17 => KeyType::Character(0x69),                   // I
            0x24..=0x26 => KeyType::Character(scancode + 0x46), // J-L
            0x32 => KeyType::Character(0x6D),                   // M
            0x31 => KeyType::Character(0x6E),                   // N
            0x18 => KeyType::Character(0x6F),                   // O
            0x19 => KeyType::Character(0x70),                   // P
            0x10 => KeyType::Character(0x71),                   // Q
            0x13 => KeyType::Character(0x72),                   // R
            0x1F => KeyType::Character(0x73),                   // S
            0x14 => KeyType::Character(0x74),                   // T
            0x16 => KeyType::Character(0x75),                   // U
            0x2F => KeyType::Character(0x76),                   // V
            0x11 => KeyType::Character(0x77),                   // W
            0x2D => KeyType::Character(0x78),                   // X
            0x15 => KeyType::Character(0x79),                   // Y
            0x2C => KeyType::Character(0x7A),                   // Z
            0x39 => KeyType::Character(0x20),                   // Space
            0x36 => {
                self.shift_enabled = state == KeyState::Pressed;
                KeyType::Action(Action::Shift)
            } // right shift
            0x2A => {
                self.shift_enabled = state == KeyState::Pressed;
                KeyType::Action(Action::Shift)
            } // left shift
            0x0E => KeyType::Action(Action::Delete),            // Delete
            0x1C => KeyType::Action(Action::Enter),             // Enter
            _ => KeyType::Undefined(scancode),
        };

        if let KeyType::Character(chr) = &mut key
            && self.shift_enabled
        {
            *chr -= 0x20;
        }

        KeyPressed { key, state }
    }
}
