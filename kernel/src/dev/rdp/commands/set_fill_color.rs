// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP Command - Set Fill Color

// TODO: Pixel and color representation

use proc_bitfield::bitfield;

bitfield! {

    /// Fill commands will draw using the color set via this command.
    ///
    pub struct SetFillColor(pub u64): FromRaw, IntoRaw {

        /// 0x37
        pub opcode: u8 @ 56..=63,

        /// Value of the fill color dependent upon the set canvas color model
        /// and pixel size. Allows packing color values smaller than a word.
        pub packed_color: u32 @ 0..=31,

    }

}

// eof
