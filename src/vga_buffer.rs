use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// A VGA text mode color code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8) & 0x0f)
    }
}

/// A VGA text mode character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl core::ops::Deref for ScreenChar {
    type Target = ScreenChar;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl core::ops::DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Write a byte to VGA buffer.
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        };
    }

    /// Shift all lines up by one, and clear the last line.
    fn new_line(&mut self) {
        for row in 0..BUFFER_HEIGHT - 1 {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(self.buffer.chars[row + 1][col].read());
            }
        }

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col].write(ScreenChar::default());
        }

        self.column_position = 0;
    }

    /// Write a printable ASCII string to VGA buffer.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x0A | 0x20..=0x7E => self.write_byte(byte),
                _ => self.write_byte(0xfe), // ■ in CP437
            }
        }
    }
}

pub fn print_test() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Cyan, Color::LightCyan),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_string(
        "Hello VGA buffer!
                 
      \\    /\\    
       )  ( ')   
      (  /  )    
       \\(__)|    ",
    );
}
