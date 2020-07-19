pub mod bochs;

use crate::ok;

use bochs::BochsGraphicsAdapter;
use lib_kern::{gfx::Command, video::VideoDevice};

use crate::arch::task::mouse::MousePacketStream;

#[allow(unused)]
pub async fn init() {
    let bga_dev = bochs::BochsGraphicsAdapter::detect().unwrap();
    let mut bga = BochsGraphicsAdapter::new(&bga_dev).init();
    print!("[BGA @ 0x{:08x}] Found", bga.addr());
    ok!();
    println!(
        "[BGA @ 0x{:08x}] Version: 0x{:04x}",
        bga.addr(),
        bga.version()
    );
    println!("[BGA @ 0x{:08x}] Max BPP: {}", bga.addr(), bga.max_bpp);
    println!("[BGA @ 0x{:08x}] Max Width: {}", bga.addr(), bga.max_width);
    println!(
        "[BGA @ 0x{:08x}] Max Height: {}",
        bga.addr(),
        bga.max_height
    );
    let mode = bga
        .get_default_mode()
        .and_then(|mode| {
            println!(
                "[BGA @ 0x{:08x}] Supports resolution: {}x{}x{}",
                bga.addr(),
                mode.width,
                mode.height,
                mode.bpp
            );
            Some(mode)
        })
        .unwrap();
    bga.set_video_mode(&mode, true);

    let mut curx: i32 = 200;
    let mut cury: i32 = 200;

    let mut video = VideoDevice::new(&bga, &mode);
    //let font_bytes = include_bytes!("FiraCode-Regular.ttf");
    //let font = video.load_font_from_bytes("FiraCode".to_string(), font_bytes);

    let mut mouse = MousePacketStream::new();

    loop {
        video.push(Command::Clear { color: 0xFF6495ED });

        video.flush();
        x86_64::instructions::hlt();
    }
}
