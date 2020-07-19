pub mod common;
pub mod font;
pub mod rect;

use super::video::VideoMode;

use alloc::{boxed::Box, collections::BTreeMap, fmt::Debug, string::String, vec::Vec};

use font::Font;

pub type CommandBuffer = Vec<Command>;

pub trait FillDebug: FillShape + Debug {}
impl FillDebug for rect::Rect {}
pub trait OutlineDebug: OutlineShape + Debug {}
impl OutlineDebug for rect::Rect {}

#[derive(Debug)]
pub enum Command {
    FillShape {
        color: u32,
        shape: Box<dyn FillDebug>,
    },
    OutlineShape {
        color: u32,
        shape: Box<dyn OutlineDebug>,
    },
    Text {
        color: u32,
        text: String,
        font: String,
        v_size: f32,
        h_size: f32,
        x_pos: i32,
        y_pos: i32,
    },
    Clear {
        color: u32,
    },
}

impl Command {
    pub fn execute(
        &self,
        font_cache: &BTreeMap<String, Font>,
        buffer: &mut Vec<u32>,
        mode: &VideoMode,
    ) {
        match self {
            Self::FillShape { color, shape } => shape.fill(*color, buffer, mode),
            Self::OutlineShape { color, shape } => shape.outline(*color, buffer, mode),
            Self::Clear { color } => rect::Rect {
                x: 0,
                y: 0,
                w: mode.width as i32,
                h: mode.height as i32,
            }
            .fill(*color, buffer, mode),
            Self::Text {
                color,
                font,
                text,
                v_size,
                h_size,
                x_pos,
                y_pos,
            } => {
                let font = font_cache.get(font).expect("cannot find font");
                let layout = font.layout(text.as_str(), (*v_size, *h_size));
                layout.paint_at(*x_pos, *y_pos, *color, buffer, mode);
            }
        }
    }
}

pub trait FillShape {
    fn fill(&self, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode);
}

pub trait OutlineShape {
    fn outline(&self, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode);
}
