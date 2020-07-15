pub mod bochs;

use crate::ok;

use bochs::BochsGraphicsAdapter;
use lib_kern::{gfx::Command, video::VideoDevice};

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

    video.push(Command::Clear { color: 0xFF6495ED });

    use lib_kern::dom::{
        color::Color,
        css::{Declaration, Rule, Selector, SimpleSelector, Stylesheet, Unit, Value},
        layout::{layout_tree, Dimensions, Rect},
        painting::build_command_buffer,
        parser::{css, html},
        style::style_tree,
    };

    let root = html::parse(
        r#"
<div class="a">
    <div class="b">
        <div class="c">
        </div>
    </div>
</div>
    "#
        .to_string(),
    );

    println!("{:#?}", root);

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
            declarations: vec![
                Declaration {
                    name: "background".to_string(),
                    value: Value::ColorValue(Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    }),
                },
                Declaration {
                    name: "border-width".to_string(),
                    value: Value::Length(2.0, Unit::Px),
                },
                Declaration {
                    name: "border-color".to_string(),
                    value: Value::ColorValue(Color {
                        r: 0,
                        g: 255,
                        b: 0,
                        a: 255,
                    }),
                },
            ],
        },
        Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec!["b".to_string()],
            })],
            declarations: vec![
                Declaration {
                    name: "background".to_string(),
                    value: Value::ColorValue(Color {
                        r: 0,
                        g: 255,
                        b: 0,
                        a: 255,
                    }),
                },
                Declaration {
                    name: "border-width".to_string(),
                    value: Value::Length(2.0, Unit::Px),
                },
                Declaration {
                    name: "border-color".to_string(),
                    value: Value::ColorValue(Color {
                        r: 0,
                        g: 0,
                        b: 255,
                        a: 255,
                    }),
                },
            ],
        },
        Rule {
            selectors: vec![Selector::Simple(SimpleSelector {
                tag_name: None,
                id: None,
                class: vec!["c".to_string()],
            })],
            declarations: vec![
                Declaration {
                    name: "background".to_string(),
                    value: Value::ColorValue(Color {
                        r: 0,
                        g: 0,
                        b: 255,
                        a: 255,
                    }),
                },
                Declaration {
                    name: "border-width".to_string(),
                    value: Value::Length(2.0, Unit::Px),
                },
                Declaration {
                    name: "border-color".to_string(),
                    value: Value::ColorValue(Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    }),
                },
            ],
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
    let styled_root = style_tree(&root, &stylesheet, &alloc::collections::BTreeMap::new());
    let layout_root = layout_tree(&styled_root, screen_block);

    let command_buffer = build_command_buffer(&layout_root);
    video.push_many(command_buffer);

    let font_bytes = include_bytes!("FiraCode-Regular.ttf");
    let font = video.load_font_from_bytes("FiraCode".to_string(), font_bytes);
    video.push(Command::Text {
        font: "FiraCode".to_string(),
        text: "Hello World".to_string(),
        h_size: 72.0,
        v_size: 72.0,
        x_pos: 0,
        y_pos: 0,
    });

    video.flush();
}
