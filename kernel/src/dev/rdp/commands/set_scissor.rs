// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP Command - Set Scissor

use proc_bitfield::bitfield;

// TODO: 10.2 format

bitfield! {

    /// Set the "scissor box" with respect to the set canvas.
    ///
    pub struct SetScissor(pub u64): FromRaw, IntoRaw {

        /// 0x37
        pub opcode: u8 @ 56..=61,

        /// X coordinate, upper left of the scissor box (relative to the canvas) in 10.2 fixed-point format
        pub x_upper_left: u16 @ 44..=55,

        /// Y coordinate, upper left of the scissor box (relative to the canvas) in 10.2 fixed-point format
        pub y_upper_left: u16 @ 32..=43,

        /// If set, odd or even lines (determined by 'interfaced_lines') will be scissored
        pub interlaced_scissoring: bool @ 25,

        /// If interlaced scissoring is enabled, picks which lines will be scissored (odd or even)
        pub scissor_line_skip: bool [ScissorLineSkip] @ 24,

        /// X coordinate, lower right of the scissor box (relative to the canvas) in 10.2 fixed-point format
        pub x_lower_right: u16 @ 12..=23,

        /// Y coordinate, lower right of the scissor box (relative to the canvas) in 10.2 fixed-point format
        pub y_lower_right: u16 @ 0..=11,

    }

}

#[allow(non_camel_case_types)]
pub enum ScissorLineSkip {

    /// Skip odd lines
    SKIP_ODD_LINES,

    /// Skip even lines
    SKIP_EVEN_LINES,

}

impl From<bool> for ScissorLineSkip {
    fn from(value: bool) -> Self {
        match value {
            true => Self::SKIP_EVEN_LINES,
            false => Self::SKIP_ODD_LINES,
        }
    }
}

impl Into<bool> for ScissorLineSkip {
    fn into(self) -> bool {
        match self {
            Self::SKIP_ODD_LINES => false,
            Self::SKIP_EVEN_LINES => true,
        }
    }
}

// eof
