use alloc::{boxed::Box, vec, vec::Vec};

use crate::gfx::{Command, CommandBuffer};

pub trait GraphicsProvider {
    fn get_framebuffer(&self, mode: &VideoMode) -> Box<&mut [u32]>;
}

pub struct VideoDevice<'a, T>
where
    T: GraphicsProvider,
{
    pub provider: &'a T,
    pub mode: VideoMode,
    pub buffer: Vec<u32>,
    pub command_buffer: CommandBuffer,
}

impl<'a, T> VideoDevice<'a, T>
where
    T: GraphicsProvider,
{
    pub fn new(provider: &'a T, mode: &VideoMode) -> VideoDevice<'a, T> {
        Self {
            provider,
            mode: mode.clone(),
            buffer: vec![0u32; mode.width * mode.height],
            command_buffer: Vec::new(),
        }
    }

    pub fn push(&mut self, command: Command) {
        self.command_buffer.push(command);
    }

    pub fn push_many(&mut self, commands: CommandBuffer) {
        self.command_buffer.extend(commands);
    }

    pub fn clear(&mut self, color: u32) {
        self.buffer.fill(color);
    }

    pub fn flush(&mut self) {
        for command in self.command_buffer.iter() {
            command.execute(&mut self.buffer, &self.mode);
        }
        self.command_buffer.clear();

        let fb: Box<&mut [u32]> = self.provider.get_framebuffer(&self.mode);
        fb.copy_from_slice(&self.buffer);
    }
}

#[derive(Debug, Clone)]
pub struct VideoMode {
    pub width: usize,
    pub height: usize,
    pub bpp: u16,
}
