// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP Command - Fill Rectangle

// TODO: 2.10 fixed-point format

use proc_bitfield::bitfield;

bitfield! {

    /// Command the RDP to draw a rectangle filled with the set color to the set
    /// canvas at the given location. The fill color, canvas, and success depend
    /// on prior submission of specific commands; see the docs.
    ///
    pub struct FillRectangle(pub u64): FromRaw, IntoRaw {

        /// 0x36
        pub opcode: u8 @ 56..=61,

        /// X coordinate, lower right of the rectangle, in 10.2 fixed-point format.
        pub x_lower_right: u16 @ 44..=55,

        /// Y coordinate, lower right of the rectangle, in 10.2 fixed-point format.
        pub y_lower_right: u16 @ 32..=43,

        /// X coordinate, upper left of the rectangle, in 10.2 fixed-point format.
        pub x_upper_left: u16 @ 12..=23,

        /// Y coordinate, upper left of the rectangle, in 10.2 fixed-point format.
        pub y_upper_left: u16 @ 0..=11,

    }

}

// eof
