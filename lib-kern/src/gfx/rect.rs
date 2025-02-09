use super::{
    common::{line, plot, Point},
    FillShape, OutlineShape, VideoMode,
};

use alloc::vec::Vec;

#[derive(Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl OutlineShape for Rect {
    fn outline(&self, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode) {
        let tl = Point {
            x: self.x,
            y: self.y,
        };
        let bl = Point {
            x: self.x,
            y: self.y + self.h,
        };
        let tr = Point {
            x: self.x + self.w,
            y: self.y,
        };
        let br = Point {
            x: self.x + self.w,
            y: self.y + self.h,
        };

        line(&tl, &tr, color, buffer, &mode);
        line(&tr, &br, color, buffer, &mode);
        line(&br, &bl, color, buffer, &mode);
        line(&bl, &tl, color, buffer, &mode);
    }
}

impl FillShape for Rect {
    fn fill(&self, color: u32, buffer: &mut Vec<u32>, mode: &VideoMode) {
        for y in self.y..self.y + self.h {
            for x in self.x..self.x + self.w {
                plot(&Point { x, y }, color, buffer, &mode)
            }
        }
    }
}
