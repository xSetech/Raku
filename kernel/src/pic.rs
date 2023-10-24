// SPDX-License-Identifier: GPL-3.0-or-later

//! Supporting structures for drawing

use proc_bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct RGBA(pub u32): FromRaw, IntoRaw {
        pub red: u8 @ 24; 8,
        pub green: u8 @ 16; 8,
        pub blue: u8 @ 8; 8,
        pub alpha: u8 @ 0; 8,
    }
}

// eof
