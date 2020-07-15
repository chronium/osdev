use super::{
    color::Color,
    css::Value,
    layout::{BoxType, LayoutBox, Rect},
};

use crate::gfx::{rect::Rect as gfxRect, Command, CommandBuffer};

use alloc::vec::Vec;

pub fn build_command_buffer(layout_root: &LayoutBox) -> CommandBuffer {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut CommandBuffer, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    // TODO: Render Text

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut CommandBuffer, layout_box: &LayoutBox) {
    let rect: gfxRect = layout_box.dimensions.border_box().into();
    get_color(layout_box, "background").map(|color| {
        list.push(Command::FillShape {
            shape: box rect,
            color: color.into(),
        })
    });
}

fn render_borders(list: &mut CommandBuffer, layout_box: &LayoutBox) {
    let color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return,
    };

    let d = &layout_box.dimensions;
    let border_box = d.border_box();

    let lb: gfxRect = Rect {
        x: border_box.x,
        y: border_box.y,
        width: d.border.left,
        height: border_box.height,
    }
    .into();
    list.push(Command::FillShape {
        color: color.into(),
        shape: box lb,
    });

    let rb: gfxRect = Rect {
        x: border_box.x + border_box.width - d.border.right,
        y: border_box.y,
        width: d.border.right,
        height: border_box.height,
    }
    .into();
    list.push(Command::FillShape {
        color: color.into(),
        shape: box rb,
    });

    let tb: gfxRect = Rect {
        x: border_box.x,
        y: border_box.y,
        width: border_box.width,
        height: d.border.top,
    }
    .into();
    list.push(Command::FillShape {
        color: color.into(),
        shape: box tb,
    });

    let bb: gfxRect = Rect {
        x: border_box.x,
        y: border_box.y + border_box.height - d.border.bottom,
        width: border_box.width,
        height: d.border.bottom,
    }
    .into();
    list.push(Command::FillShape {
        color: color.into(),
        shape: box bb,
    });
}

fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => match style.value(name) {
            Some(Value::ColorValue(color)) => Some(color),
            _ => None,
        },
        BoxType::AnonymousBlock => None,
    }
}
