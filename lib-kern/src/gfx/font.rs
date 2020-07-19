use rusttype::{point, Font as rtFont, PositionedGlyph, Scale, VMetrics};

use alloc::vec::Vec;

use super::common;
use crate::video::VideoMode;

#[derive(Debug)]
pub struct Font<'a> {
    inner: rtFont<'a>,
}

impl<'a> Font<'a> {
    fn new(font: rtFont<'a>) -> Self {
        Self { inner: font }
    }

    pub fn try_from_bytes(bytes: &'a [u8]) -> Option<Self> {
        Some(Font::new(rtFont::try_from_bytes(bytes)?))
    }

    pub fn layout(&self, str: &str, (w, h): (f32, f32)) -> Layout {
        Layout::new(&self.inner, str, w, h)
    }
}

#[allow(unused)]
pub struct Layout<'a> {
    inner: Vec<PositionedGlyph<'a>>,
    v_metrics: VMetrics,
    scale: Scale,
}

macro_rules! clamp {
    ($a:expr, $min:expr, $max:expr) => {{
        if $a < $min {
            $min
        } else if $a > $max {
            $max
        } else {
            $a
        }
    }};
}

macro_rules! gamma_correct {
    ($value:expr) => {{
        clamp!(libm::powf($value, 1.0 / 2.0) * 255.0, 0.0, 255.0) as u8
    }};
}

impl<'a> Layout<'a> {
    fn new(font: &'a rtFont, str: &str, x: f32, y: f32) -> Self {
        let scale = Scale { x, y };
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        let glyphs = font.layout(str, scale, offset);
        Self {
            inner: glyphs.collect(),
            v_metrics,
            scale,
        }
    }

    pub fn width(&self) -> f32 {
        self.inner
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
    }

    pub fn paint_at(
        &self,
        x_pos: i32,
        y_pos: i32,
        color: u32,
        buffer: &mut Vec<u32>,
        mode: &VideoMode,
    ) {
        for g in &self.inner {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let x = x as i32 + bb.min.x + x_pos;
                    let y = y as i32 + bb.min.y + y_pos;

                    common::plot(
                        &common::Point { x, y },
                        ((gamma_correct!(v) as u32) << 24) | (color & 0xFFFFFF),
                        buffer,
                        mode,
                    );
                })
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    (if x.is_nan() || x < y { y } else { x }) * 1.0
}

#[no_mangle]
pub extern "C" fn fminf(x: f32, y: f32) -> f32 {
    (if y.is_nan() || x < y { x } else { y }) * 1.0
}
