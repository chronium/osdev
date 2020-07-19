use pic8259_simple::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFS: u8 = 32;
pub const PIC_2_OFFS: u8 = PIC_1_OFFS + 8;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFS,
    Keyboard,
    Mouse = PIC_1_OFFS + 12,
}

impl From<InterruptIndex> for u8 {
    fn from(ii: InterruptIndex) -> u8 {
        ii as u8
    }
}

impl From<InterruptIndex> for usize {
    fn from(ii: InterruptIndex) -> usize {
        ii as usize
    }
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFS, PIC_2_OFFS) });

pub fn init() {
    use crate::ok;
    unsafe {
        PICS.lock().initialize();
    }
    print!("PICS loaded");
    ok!();
}
