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
    let pt = pt.x + pt.y * mode.width as isize;
    assert!(!pt.is_negative());
    buffer[pt as usize] = color;
}

pub struct Point {
    pub x: isize,
    pub y: isize,
}
