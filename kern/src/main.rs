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
pub mod log;
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
    use alloc::string::ToString;
    println!("\nSCHEMAS");
    check_ok!(
        "Registering sys schema",
        SCHEMA_MAP
            .lock()
            .register("sys".to_string(), schema::sys::SysSchema::new())
    );
}

async fn dump() {
    println!("\nDumping devices + schemas");
    for dev in DEVICE_MAP.lock().dump_names() {
        println!("Device {}", dev);
    }
    for dev in SCHEMA_MAP.lock().inner().dump_names() {
        println!("Schema {}", dev);
    }
    println!("\n");

    println!("find: {:?}", SCHEMA_MAP.lock().find("sys://info"));
    let info = SCHEMA_MAP.lock().open("sys://info");
    println!("open: {:?}", info);
    let info = info.unwrap();
    use alloc::{string::String, vec::Vec};
    let mut buf = Vec::new();
    info.read(&mut buf).ok();
    println!("read: {:?}", String::from_utf8(buf));
    println!("close: {:?}", info.close());
}

use lazy_static::lazy_static;
use lib_kern::{io::DeviceMap, schema::driver::SchemaDriver};
lazy_static! {
    static ref DEVICE_MAP: Mutex<DeviceMap> = Mutex::new(DeviceMap::new());
    static ref SCHEMA_MAP: Mutex<SchemaDriver> = Mutex::new(SchemaDriver::new());
}

#[panic_handler]
fn painc(info: &PanicInfo) -> ! {
    println!("{}", info);
    arch::hlt_loop()
}
