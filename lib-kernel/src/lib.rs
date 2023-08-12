//! Kernel library

#![no_std]

pub const VIDEO_INTERFACE_CONTROL: usize = 0xA4400000;
pub const FRAMEBUFFER_1_ADDRESS: usize = 0xA0100000;
pub const FRAMEBUFFER_2_ADDRESS: usize = 0xA014B000;
