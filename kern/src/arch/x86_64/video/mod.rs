pub mod bochs;

use crate::ok;

use bochs::BochsGraphicsAdapter;
use lib_kern::{
    gfx::{Command, FillShape, OutlineShape},
    video::VideoDevice,
};

use alloc::{string::ToString, vec};

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

    use lib_kern::dom::{
        css::{Color, Declaration, Rule, Selector, SimpleSelector, Stylesheet, Unit, Value},
        elem,
        layout::{layout_tree, Dimensions, Rect},
        painting::build_command_buffer,
        style::style_tree,
        AttrMap,
    };

    let mut am = AttrMap::new();
    am.insert("class".to_string(), "a".to_string());
    let mut bm = AttrMap::new();
    bm.insert("class".to_string(), "b".to_string());
    let mut cm = AttrMap::new();
    cm.insert("class".to_string(), "c".to_string());
    let root = elem(
        "div".to_string(),
        am,
        vec![elem(
            "div".to_string(),
            bm,
            vec![elem("div".to_string(), cm, vec![])],
        )],
    );

    let rules = vec![
        Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec![],
            })],
            declarations: vec![
                Declaration {
                    name: "display".to_string(),
                    value: Value::Keyword("block".to_string()),
                },
                Declaration {
                    name: "padding".to_string(),
                    value: Value::Length(12.0, Unit::Px),
                },
            ],
        },
        Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec!["a".to_string()],
            })],
            declarations: vec![Declaration {
                name: "background".to_string(),
                value: Value::ColorValue(Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 0,
                }),
            }],
        },
        Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec!["b".to_string()],
            })],
            declarations: vec![Declaration {
                name: "background".to_string(),
                value: Value::ColorValue(Color {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 0,
                }),
            }],
        },
        Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec!["c".to_string()],
            })],
            declarations: vec![Declaration {
                name: "background".to_string(),
                value: Value::ColorValue(Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 0,
                }),
            }],
        },
    ];

    let screen_block = Dimensions {
        content: Rect {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 720.0,
        },
        padding: Default::default(),
        border: Default::default(),
        margin: Default::default(),
    };

    let stylesheet = Stylesheet { rules };
    let styled_root = style_tree(&root, &stylesheet);
    let layout_root = layout_tree(&styled_root, screen_block);

    let command_buffer = build_command_buffer(&layout_root);
    video.push_many(command_buffer);

    video.flush();
}
