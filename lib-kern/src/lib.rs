#![no_std]
#![feature(box_syntax, slice_fill, core_intrinsics)]

pub mod ansi;
pub mod gfx;
pub mod io;
pub mod schema;
pub mod video;

extern crate alloc;
