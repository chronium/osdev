#[macro_use]
pub mod print;
pub mod gdt;
pub mod idt;
pub mod mem;
pub mod pci;
pub mod pic;
pub mod task;
pub mod vga_text;
pub mod video;

use lazy_static::lazy_static;
use lib_kern::io::{CharDevice, ReadWrite};
use spinning::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::port::Port;

use core::fmt;

use self::vga_text::{Buffer, Color, Writer};

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        col: 0,
        row: 0,
        addr: Port::new(0x03D4),
        data: Port::new(0x03D5),
        fg: Color::LightGray as u8,
        bg: Color::Black as u8,
        def_fg: Color::LightGray as u8,
        def_bg: Color::Black as u8,
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub struct SerialDevice(pub u8);

impl CharDevice for SerialDevice {
    fn write_u8(&mut self, val: u8) {
        match self.0 {
            0 => SERIAL1.lock().send(val),
            _ => unreachable!(),
        }
    }

    fn write_str(&mut self, val: &str) {
        for b in val.bytes() {
            self.write_u8(b);
        }
    }

    fn get_rw(&self) -> ReadWrite {
        ReadWrite::ReadOnly
    }
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
        SERIAL1.lock().write_fmt(args).unwrap();
    });
}

pub fn init() {
    gdt::init();
    task::mouse::init();
    idt::init();
    pic::init();

    x86_64::instructions::interrupts::enable();
}
