// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP Command - Full Sync / "Sync Full"

use proc_bitfield::bitfield;

bitfield! {

    /// Command the RDP to finish processing all prior commands.
    ///
    pub struct FullSync(pub u64): FromRaw, IntoRaw {

        /// 0x29
        pub opcode: u8 @ 56..=63,

    }

}

// eof
