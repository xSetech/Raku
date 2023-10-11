// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP Command - Set Color Image

use num_enum::{FromPrimitive, IntoPrimitive};
use proc_bitfield::bitfield;

bitfield! {

    /// Defines the canvas to which the RDP will draw.
    ///
    pub struct SetColorImage(pub u64): FromRaw, IntoRaw {

        /// 0x3F
        pub opcode: u8 @ 56..=61,

        /// Color model of data drawn to the canvas (e.g. RGBA)
        pub model: u8 [CanvasColorModel] @ 53..=55,

        /// Total size, in bits, of a pixel within the canvas
        pub pixel_size: u8 [CanvasPixelSize] @ 51..=52,

        /// Width of the canvas, in pixels
        pub width: u16 @ 32..=41,

        /// Address of the first pixel of the canvas, at the top-left corner
        pub address: u32 @ 0..=25,

    }

}

/// Color model of data drawn to the canvas by the RDP
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CanvasColorModel {

    #[default]
    RGBA = 0b000,

}

/// Color model of data drawn to the canvas by the RDP
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CanvasPixelSize {

    /// 32-bits, RGBA
    #[default]
    WORD = 0b11,

}

// eof
