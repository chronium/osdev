use alloc::{boxed::Box, collections::BTreeMap, string::String, vec, vec::Vec};

use crate::gfx::{font::Font, Command, CommandBuffer};

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
    pub font_cache: BTreeMap<String, Font<'a>>,
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
            font_cache: BTreeMap::new(),
        }
    }

    pub fn load_font_from_bytes(&mut self, name: String, bytes: &'a [u8]) -> Option<()> {
        self.font_cache.insert(name, Font::try_from_bytes(bytes)?);
        Some(())
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
            command.execute(&self.font_cache, &mut self.buffer, &self.mode);
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
