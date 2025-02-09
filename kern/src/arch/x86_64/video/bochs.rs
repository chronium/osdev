use crate::arch::pci::{PCIDevice, PCIFind, PCIBAR};

use alloc::boxed::Box;
use core::{ptr::Unique, slice};
use lazy_static::lazy_static;
use lib_kern::video::{GraphicsProvider, VideoMode};

lazy_static! {
    pub static ref BGA_SIGNATURE: PCIFind = PCIFind::new(0x1234, 0x1111);
    static ref DEFAULT_VIDEO_MODE: VideoMode = VideoMode {
        width: 800,
        height: 600,
        bpp: 32
    };
}

#[allow(unused)]
mod registers {
    pub const VBE_DISPI_GETCAPS: u16 = 2;
    pub const VBE_DISPI_NUM_REGISTERS: u16 = 10;

    pub const VBE_DISPI_INDEX_ID: u16 = 0;
    pub const VBE_DISPI_INDEX_XRES: u16 = 1;
    pub const VBE_DISPI_INDEX_YRES: u16 = 2;
    pub const VBE_DISPI_INDEX_BPP: u16 = 3;
    pub const VBE_DISPI_INDEX_ENABLE: u16 = 4;

    pub const VBE_DISPI_DISABLED: u16 = 0;
    pub const VBE_DISPI_ENABLED: u16 = 1;

    pub const VBE_DISPI_LFB_ENABLED: u16 = 64;
    pub const VBE_DISPI_NOCLEAR: u16 = 128;
}

#[allow(unused)]
pub struct BochsGraphicsAdapter {
    pci_device: PCIDevice,
    pub max_bpp: u16,
    pub max_width: usize,
    pub max_height: usize,
    framebuffer_bar: PCIBAR,
    mmio_bar: PCIBAR,
    registers: Unique<[u16; registers::VBE_DISPI_NUM_REGISTERS as usize]>,
}

impl GraphicsProvider for BochsGraphicsAdapter {
    fn get_framebuffer(&self, mode: &VideoMode) -> Box<&mut [u32]> {
        let size: usize = (mode.width * mode.height) as usize;
        unsafe {
            let slice = slice::from_raw_parts_mut(self.framebuffer_bar.addr() as *mut u32, size);
            box slice
        }
    }
}

#[allow(unused)]
impl BochsGraphicsAdapter {
    pub fn new(dev: &PCIDevice) -> Self {
        let fb_bar = dev.get_bar(0);
        let mmio_bar = dev.get_bar(2);
        let mmio = mmio_bar.addr();

        fb_bar
            .identity_map()
            .expect("Unable to map BGA framebuffer");
        mmio_bar.identity_map().expect("Unable to map BGA MMIO");

        Self {
            pci_device: *dev,
            max_bpp: 0,
            max_width: 0,
            max_height: 0,
            framebuffer_bar: fb_bar,
            mmio_bar,
            registers: Unique::new((mmio + 0x500) as *mut _).unwrap(),
        }
    }

    pub fn addr(&self) -> u32 {
        u32::from(self.pci_device.address)
    }

    pub fn version(&self) -> u16 {
        self.read_reg(registers::VBE_DISPI_INDEX_ID)
    }

    pub fn init(mut self) -> Self {
        let max_bpp = self.get_capability(registers::VBE_DISPI_INDEX_BPP);
        let max_width = self.get_capability(registers::VBE_DISPI_INDEX_XRES);
        let max_height = self.get_capability(registers::VBE_DISPI_INDEX_YRES);

        self.max_bpp = max_bpp;
        self.max_width = max_width as usize;
        self.max_height = max_height as usize;

        self
    }

    pub fn set_video_mode(&mut self, mode: &VideoMode, clear: bool) {
        let mut enable = registers::VBE_DISPI_ENABLED | registers::VBE_DISPI_LFB_ENABLED;
        if !clear {
            enable |= registers::VBE_DISPI_NOCLEAR;
        }

        self.write_reg(
            registers::VBE_DISPI_INDEX_ENABLE,
            registers::VBE_DISPI_DISABLED,
        );
        self.write_reg(registers::VBE_DISPI_INDEX_XRES, mode.width as u16);
        self.write_reg(registers::VBE_DISPI_INDEX_YRES, mode.height as u16);
        self.write_reg(registers::VBE_DISPI_INDEX_BPP, mode.bpp);
        self.write_reg(registers::VBE_DISPI_INDEX_ENABLE, enable);
    }

    pub fn get_default_mode(&self) -> Option<VideoMode> {
        if self.supports_resolution(DEFAULT_VIDEO_MODE.clone()) {
            return Some(DEFAULT_VIDEO_MODE.clone());
        }
        None
    }

    pub fn supports_resolution(&self, mode: VideoMode) -> bool {
        if mode.width > self.max_width || mode.height > self.max_height || mode.bpp > self.max_bpp {
            return false;
        }
        true
    }

    fn read_reg(&self, index: u16) -> u16 {
        assert!(index < registers::VBE_DISPI_NUM_REGISTERS);
        unsafe { self.registers.as_ref()[index as usize] }
    }

    fn write_reg(&mut self, index: u16, val: u16) {
        assert!(index < registers::VBE_DISPI_NUM_REGISTERS);
        unsafe { self.registers.as_mut()[index as usize] = val };
    }

    fn get_capability(&mut self, index: u16) -> u16 {
        let was_enabled = self.read_reg(registers::VBE_DISPI_INDEX_ENABLE);
        self.write_reg(
            registers::VBE_DISPI_INDEX_ENABLE,
            was_enabled | registers::VBE_DISPI_GETCAPS,
        );
        let cap = self.read_reg(index);
        assert!(cap != 0); // Someone, if you can find why this is needed, please tell me. I'm desperate
        self.write_reg(registers::VBE_DISPI_INDEX_ENABLE, was_enabled);
        cap
    }

    pub fn detect() -> Result<PCIDevice, &'static str> {
        PCIDevice::search(&BGA_SIGNATURE, None).ok_or("Could not find Bochs Graphics Adapter")
    }
}
