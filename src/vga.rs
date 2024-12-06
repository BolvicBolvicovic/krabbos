use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::tables::port::Port;

const   VGA_BUFFER_ADDR: *mut VGABuffer = 0xB8000 as *mut VGABuffer;
const   VGA_BUFFER_HEIGHT: usize        = 25;
const   VGA_BUFFER_WIDTH: usize         = 80;
const   VGA_OFFSET_LOW: usize	        = 0x0F;
const   VGA_OFFSET_HIGH: usize	        = 0x0E;

lazy_static! {
    pub static ref VGA_WRITER: Mutex<VGAWriter> = {
        let w = Mutex::new(VGAWriter {
            column_pos: 0,
            row_pos: 0,
            color_code: VGAColorCode::new(VGAColor::BrightWhite, VGAColor::Black),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR) }
        });
        w.lock().update_colors(VGAColor::BrightWhite, VGAColor::Black);
        w
    };

    static ref VGA_CRTL_PORT: Mutex<Port> = Mutex::new(Port::new(0x3D4));
    static ref VGA_DATA_PORT: Mutex<Port> = Mutex::new(Port::new(0x3D5));
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VGAColor {
    Black           = 0,
    Blue            = 1,
    Green           = 2,
    Cyan            = 3,
    Red             = 4,
    Magenta         = 5,
    Brown           = 6,
    White           = 7,
    Gray            = 8,
    LightBlue       = 9,
    LightGreen      = 10,
    LightCyan       = 11,
    LightRed        = 12,
    LightMagenta    = 13,
    Yellow          = 14,
    BrightWhite     = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct VGAColorCode(u8);

impl VGAColorCode {
    fn new(fg: VGAColor, bg: VGAColor) -> Self {
        VGAColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VGAChar {
    ascii_character: u8,
    color_code: VGAColorCode,
}

#[repr(transparent)]
struct VGABuffer {
    chars: [[VGAChar; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGHT]
}

pub struct VGAWriter {
    column_pos: usize,
    row_pos: usize,
    color_code: VGAColorCode,
    buffer: &'static mut VGABuffer,
}

impl VGAWriter {
    pub fn update_colors(&mut self, fg: VGAColor, bg: VGAColor) {
        let color_code: VGAColorCode = VGAColorCode::new(fg, bg);
        self.color_code = color_code;
        for x in 0..VGA_BUFFER_HEIGHT {
            for y in 0..VGA_BUFFER_WIDTH {
                self.buffer.chars[x][y].color_code = self.color_code;
            }
        }
    }

    pub fn write_string(&mut self, bytes: &str) {
        for byte in bytes.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos + 1 == VGA_BUFFER_WIDTH {
                    self.new_line();
                    self.buffer.chars[self.row_pos][self.column_pos].ascii_character = byte;
                } else {
                    self.buffer.chars[self.row_pos][self.column_pos].ascii_character = byte;
                }
                self.column_pos += 1;
            },
        }
        self.set_cursor(self.row_pos * VGA_BUFFER_WIDTH + self.column_pos);
    }

    fn new_line(&mut self) {
        if self.row_pos + 1 == VGA_BUFFER_HEIGHT {
            self.scroll();
            self.column_pos = 0;
        } else {
            self.row_pos += 1;
            self.column_pos = 0;
        }
    }

    fn scroll(&mut self) {
        for x in 1..VGA_BUFFER_HEIGHT {
            for y in 0..VGA_BUFFER_WIDTH {
                self.buffer.chars[x - 1][y] = self.buffer.chars[x][y];
            }
        }
        for x in 0..VGA_BUFFER_WIDTH {
            self.buffer.chars[VGA_BUFFER_HEIGHT - 1][x].ascii_character = b' ';
        }
    }

    fn set_cursor(&self, offset: usize) {
        unsafe {
            VGA_CRTL_PORT.lock().write(VGA_OFFSET_HIGH as u8);
            VGA_DATA_PORT.lock().write(((offset) >> 8) as u8);
            VGA_CRTL_PORT.lock().write(VGA_OFFSET_LOW as u8);
            VGA_DATA_PORT.lock().write(((offset) & 0xFF) as u8);
        }
    }
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    VGA_WRITER.lock().write_fmt(args).unwrap();
}
