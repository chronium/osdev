use lib_kern::{
    ansi::{AnsiAdapter, AnsiEscape},
    io::{CharDevice, ReadWrite},
};

use volatile::Volatile;
use x86_64::instructions::port::Port;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

macro_rules! color {
    ($fc:expr, $bc:expr) => {
        $bc << 4 | $fc
    };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct ScreenChar {
    ascii: u8,
    color: u8,
}

const BUF_HEIGHT: usize = 25;
const BUF_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUF_WIDTH]; BUF_HEIGHT],
}

pub struct Writer {
    pub(super) col: usize,
    pub(super) row: usize,
    pub(super) fg: u8,
    pub(super) bg: u8,
    pub(super) def_fg: u8,
    pub(super) def_bg: u8,
    pub(super) addr: Port<u8>,
    pub(super) data: Port<u8>,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= BUF_HEIGHT {
            for row in 1..BUF_HEIGHT {
                for col in 0..BUF_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUF_HEIGHT - 1);
            self.row = BUF_HEIGHT - 1;
        }
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii: b' ',
            color: color!(self.fg, self.bg),
        };
        for col in 0..BUF_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn update_cursor(&mut self) {
        let off = self.row * BUF_WIDTH + self.col;
        let mut addr = Port::new(0x03D4);
        let mut data = Port::new(0x03D5);
        unsafe {
            addr.write(0x0E_u8);
            data.write((off >> 0x08) as u8);
            addr.write(0x0F_u8);
            data.write((off & 0xFF) as u8);
        }
    }

    fn write_u8(&mut self, val: u8) {
        match val {
            b'\n' => {
                self.newline();
                self.update_cursor();
            }
            byte => {
                self.buffer.chars[self.row][self.col].write(ScreenChar {
                    ascii: byte,
                    color: color!(self.fg, self.bg),
                });
                self.col += 1;

                if self.col >= BUF_WIDTH {
                    self.newline();
                }
                self.update_cursor();
            }
        }
    }

    fn write_str(&mut self, val: &str) {
        let chars = val.as_bytes();
        let mut i = 0;

        'outer: while i < chars.len() {
            match chars[i] {
                b'\x1b' => {
                    i += 1;
                    if i >= chars.len() || chars[i] != b'[' {
                        break 'outer;
                    }
                    i += 1;
                    let (codes, skip) = AnsiAdapter::parse(&chars[i..]);
                    for code in codes {
                        match code {
                            None => {}
                            Some(AnsiEscape::Reset) => {
                                self.reset();
                            }
                            Some(AnsiEscape::Foreground(color)) => {
                                self.fg = color;
                            }
                            Some(AnsiEscape::Background(color)) => {
                                self.bg = color;
                            }
                        }
                    }
                    i += skip;
                }
                _ => self.write_u8(chars[i] as u8),
            }

            i += 1;
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.fg = self.def_fg;
        self.bg = self.def_bg;
    }
}

pub struct WriterDevice;

impl CharDevice for WriterDevice {
    fn write_u8(&mut self, val: u8) {
        super::WRITER.lock().write_u8(val);
    }

    fn write_str(&mut self, val: &str) {
        super::WRITER.lock().write_str(val);
    }

    fn get_rw(&self) -> ReadWrite {
        ReadWrite::WriteOnly
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
