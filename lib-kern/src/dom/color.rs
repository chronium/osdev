use core::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> u32 {
        ((color.a as u32) << 24)
            | ((color.r as u32) << 16)
            | ((color.g as u32) << 8)
            | (color.b as u32)
    }
}

impl TryFrom<&str> for Color {
    type Error = ();

    fn try_from(s: &str) -> Result<Color, Self::Error> {
        match s.to_lowercase().as_str() {
            "black" => Ok(Color::new(0, 0, 0, 255)),
            "white" => Ok(Color::new(255, 255, 255, 255)),
            "red" => Ok(Color::new(255, 0, 0, 255)),
            "green" => Ok(Color::new(0, 255, 0, 255)),
            "blue" => Ok(Color::new(0, 0, 255, 255)),
            _ => Err(()),
        }
    }
}
