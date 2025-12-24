#[derive(PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(PartialEq)]
pub struct KeyPressed {
    pub chr: u8,
    pub state: KeyState
}

pub struct Keyboard {
    pub shift_enabled: bool,
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard { shift_enabled: false }
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

    pub fn parse_scancode(&mut self, mut scancode: u8) -> KeyPressed {
        let state = if scancode & 0x80 != 0 {
            scancode -= 0x80;
            KeyState::Released
        } else {
            KeyState::Pressed
        };

        let mut chr = match scancode {
            0x0B => 0x30,                       // 0
            0x02..=0x0A => scancode + 0x2F,     // 1-9
            0x1E => 0x61,                       // A
            0x30 => 0x62,                       // B
            0x2E => 0x63,                       // C
            0x20 => 0x64,                       // D
            0x12 => 0x65,                       // E
            0x21..=0x23 => scancode + 0x45,     // F-H
            0x17 => 0x69,                       // I
            0x24..=0x26 => scancode + 0x46,     // J-L
            0x32 => 0x6D,                       // M
            0x31 => 0x6E,                       // N
            0x18 => 0x6F,                       // O
            0x19 => 0x70,                       // P
            0x10 => 0x71,                       // Q
            0x13 => 0x72,                       // R
            0x1F => 0x73,                       // S
            0x14 => 0x74,                       // T
            0x16 => 0x75,                       // U
            0x2F => 0x76,                       // V
            0x11 => 0x77,                       // W
            0x2D => 0x78,                       // X
            0x15 => 0x79,                       // Y
            0x2C => 0x7A,                       // Z
            0x36 => {
                self.shift_enabled = state == KeyState::Pressed;
                0x0E
            },                                  // right shift
            0x2A => {
                self.shift_enabled = state == KeyState::Pressed;
                0x0E
            },                                  // left shift
            _ => scancode
        };

        if self.shift_enabled && Keyboard::is_alphabet(chr) {
            chr -= 0x20;
        }

        KeyPressed {
            chr,
            state
        }
    }
}
