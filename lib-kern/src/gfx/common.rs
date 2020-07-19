use super::VideoMode;

use alloc::vec::Vec;
use core::mem::swap;

pub fn line(from: &Point, to: &Point, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode) {
    let mut y_longer = false;
    let mut short_len = to.y - from.y;
    let mut long_len = to.x - from.x;

    if short_len.abs() > long_len.abs() {
        swap(&mut short_len, &mut long_len);
        y_longer = true;
    }

    let dec_inc = if long_len == 0 {
        0
    } else {
        (short_len << 16) / long_len
    };

    if y_longer {
        let mut j = 0x8000 + (from.x << 16);
        if long_len > 0 {
            long_len += from.y;
            for y in from.y..=long_len {
                plot(&Point { x: j >> 16, y }, color, buffer, mode);
                j += dec_inc;
            }
            return;
        }
        long_len += from.y;
        for y in long_len..=from.y {
            plot(&Point { x: j >> 16, y }, color, buffer, mode);
            j -= dec_inc;
        }
        return;
    }

    let mut j = 0x8000 + (from.y << 16);
    if long_len > 0 {
        long_len += from.x;

        for x in from.x..=long_len {
            plot(&Point { x, y: j >> 16 }, color, buffer, mode);
            j += dec_inc;
        }
        return;
    }

    long_len += from.x;
    for x in long_len..=from.x {
        plot(&Point { x, y: j >> 16 }, color, buffer, mode);
        j -= dec_inc;
    }
}

#[inline(always)]
pub fn plot(pt: &Point, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode) {
    if pt.x < 0 || pt.x >= mode.width as i32 || pt.y < 0 || pt.y >= mode.height as i32 {
        return;
    }
    let pt = pt.x + pt.y * mode.width as i32;
    assert!(!pt.is_negative());
    let alpha = (((color & 0xFF000000) >> 24) as f32) / 255.0;
    if alpha == 0.0 {
        return;
    } else if alpha == 1.0 {
        buffer[pt as usize] = color & 0x00FFFFFF;
    } else {
        let src = Color::from_argb(color);
        let dst = Color::from_rgb(buffer[pt as usize]);

        let blend = src * alpha + dst * (1.0 - alpha);

        buffer[pt as usize] = blend.into();
    }
}

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn from_rgb(rgb: u32) -> Self {
        Self {
            r: ((rgb & 0xFF0000) >> 16) as u8,
            g: ((rgb & 0x00FF00) >> 8) as u8,
            b: (rgb & 0x0000FF) as u8,
            a: 255,
        }
    }

    fn from_argb(argb: u32) -> Self {
        Self {
            r: ((argb & 0x00FF0000) >> 16) as u8,
            g: ((argb & 0x0000FF00) >> 8) as u8,
            b: (argb & 0x000000FF) as u8,
            a: ((argb & 0xFF000000) >> 24) as u8,
        }
    }
}

use core::ops::{Add, Mul};

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            r: ((self.r as f32) * rhs) as u8,
            g: ((self.g as f32) * rhs) as u8,
            b: ((self.b as f32) * rhs) as u8,
            a: self.a,
        }
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Self::Output {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a,
        }
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }
}
