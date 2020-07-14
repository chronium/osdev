pub mod common;
pub mod rect;

use super::video::VideoMode;

use alloc::{boxed::Box, fmt::Debug, vec::Vec};

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
    Clear {
        color: u32,
    },
}

impl Command {
    pub fn execute(&self, buffer: &mut Vec<u32>, mode: &VideoMode) {
        match self {
            Self::FillShape { color, shape } => shape.fill(*color, buffer, mode),
            Self::OutlineShape { color, shape } => shape.outline(*color, buffer, mode),
            Self::Clear { color } => rect::Rect {
                x: 0,
                y: 0,
                w: mode.width as isize,
                h: mode.height as isize,
            }
            .fill(*color, buffer, mode),
        }
    }
}

pub trait FillShape {
    fn fill(&self, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode);
}

pub trait OutlineShape {
    fn outline(&self, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode);
}
