pub mod bochs;

use crate::ok;

use bochs::BochsGraphicsAdapter;
use lib_kern::{
    gfx::{rect::Rect, Command, FillShape, OutlineShape},
    video::VideoDevice,
};

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

    let mut video = VideoDevice::new(&bga, &mode);

    #[inline(always)]
    fn get_col(r: u8, g: u8, b: u8) -> u32 {
        (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    }

    video.push(Command::Clear { color: 0x6495ED });

    video.push(Command::FillShape {
        color: 0xFFFFFF,
        shape: box Rect {
            x: 100,
            y: 100,
            w: 100,
            h: 100,
        },
    });

    video.push(Command::OutlineShape {
        color: 0xFF0000,
        shape: box Rect {
            x: 100,
            y: 100,
            w: 100,
            h: 100,
        },
    });

    video.flush();
}
