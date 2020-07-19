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
mod schema;

extern crate alloc;

use arch::{
    mem,
    mem::paging,
    task::{executor::Executor, keyboard, mouse::MousePacketStream, Task},
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

macro_rules! check_ok {
    ($msg:expr, $val:expr) => {
        print!("{}", $msg);
        if $val.is_ok() {
            ok!();
        } else {
            fail!();
        };
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
    executor.spawn(Task::new(setup_devices()));
    executor.spawn(Task::new(setup_schemas()));
    executor.spawn(Task::new(dump()));
    //executor.spawn(Task::new(arch::video::init()));
    executor.run();
}

async fn setup_devices() {
    println!("\nDEVICES");
    check_ok!(
        "Registering tty0",
        DEVICE_MAP
            .lock()
            .insert("tty0", arch::vga_text::WriterDevice)
    );
    check_ok!(
        "Registering sty0",
        DEVICE_MAP.lock().insert("sty0", arch::SerialDevice(0))
    );

    // initialize mouse queue, to be removed
    MousePacketStream::new();
}

async fn setup_schemas() {
    println!("\nSCHEMAS");
    check_ok!(
        "Registering sys schema",
        SCHEMA_MAP
            .lock()
            .register("sys", schema::sys::SysSchema::new())
    );
}

async fn dump() {
    println!("\nDumping devices + schemas");
    for dev in DEVICE_MAP.lock().dump_names() {
        println!("Device {}", dev);
    }
}

use lazy_static::lazy_static;
use lib_kern::{io::DeviceMap, schema::SchemaMap};
lazy_static! {
    static ref DEVICE_MAP: spin::Mutex<DeviceMap> = spin::Mutex::new(DeviceMap::new());
    static ref SCHEMA_MAP: spin::Mutex<SchemaMap> = spin::Mutex::new(SchemaMap::new());
}

#[panic_handler]
fn painc(info: &PanicInfo) -> ! {
    println!("{}", info);
    arch::hlt_loop()
}
