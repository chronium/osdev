#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    alloc_error_handler,
    wake_trait,
    async_closure,
    ptr_internals,
    box_syntax
)]

#[macro_use]
mod arch;

extern crate alloc;

use arch::{
    mem,
    mem::paging,
    task::{executor::Executor, keyboard, Task},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use spinning::{Mutex, Once};
use x86_64::{structures::paging::OffsetPageTable, VirtAddr};

entry_point!(kmain);

#[macro_export]
macro_rules! ok {
    () => {
        print!(" [\x1b[32mOK\x1b[0m]\n");
    };
}

#[macro_export]
macro_rules! fail {
    () => {
        print!(" [\x1b[31mFAIL\x1b[0m]\n");
    };
}

pub static MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();
pub static FRAME_ALLOC: Once<Mutex<paging::BootInfoFrameAllocator>> = Once::new();

fn kmain(boot_info: &'static BootInfo) -> ! {
    let phys_mem_offs = VirtAddr::new(boot_info.physical_memory_offset);
    MAPPER.call_once(|| Mutex::new(unsafe { paging::init(phys_mem_offs) }));
    FRAME_ALLOC.call_once(|| {
        Mutex::new(unsafe { paging::BootInfoFrameAllocator::init(&boot_info.memory_map) })
    });
    mem::alloc::init().expect("heap initialization failed");

    print!("Serial + VGA Buffer loaded");
    ok!();
    arch::init();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(test_devices()));
    executor.spawn(Task::new(arch::video::init()));
    executor.run();
}

async fn test_devices() {
    use arch::pci::{PCIDevice, PCIFind};
    let dev = PCIDevice::search(&PCIFind::new(0x1AF4, 0x1000), None).unwrap();
    println!(
        "{:X}:{:X}:{:X}:{:X}:{:X}:{:X}",
        dev.read8(0x14),
        dev.read8(0x15),
        dev.read8(0x16),
        dev.read8(0x17),
        dev.read8(0x18),
        dev.read8(0x19)
    );

    DEVICE_MAP
        .lock()
        .insert("tty0", arch::vga_text::WriterDevice);
    DEVICE_MAP.lock().insert("sty0", arch::SerialDevice(0));

    for dev in DEVICE_MAP.lock().dump_names() {
        println!("Device {}", dev);
    }

    DEVICE_MAP.lock().get("tty0").unwrap().write_u8(b'A');
}

use lazy_static::lazy_static;
use lib_kern::io::DeviceMap;
lazy_static! {
    static ref DEVICE_MAP: spin::Mutex<DeviceMap> = spin::Mutex::new(DeviceMap::new());
}

#[panic_handler]
fn painc(info: &PanicInfo) -> ! {
    println!("{}", info);
    arch::hlt_loop()
}
