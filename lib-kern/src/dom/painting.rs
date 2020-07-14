use super::{
    css::{Color, Value},
    layout::{BoxType, LayoutBox},
};

use crate::gfx::{Command, CommandBuffer};

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
    let rect: crate::gfx::rect::Rect = layout_box.dimensions.border_box().into();
    get_color(layout_box, "background").map(|color| {
        list.push(Command::FillShape {
            shape: box rect,
            color: color.into(),
        })
    });
}

fn render_borders(list: &mut CommandBuffer, layout_box: &LayoutBox) {}

fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => match style.value(name) {
            Some(Value::ColorValue(color)) => Some(color),
            _ => None,
        },
        BoxType::AnonymousBlock => None,
    }
}
