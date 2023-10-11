// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP - Device interface and registers
//!
//! Documentation:
//!     - https://n64brew.dev/wiki/Reality_Display_Processor/Interface
//!

use crate::dev::reg::{RO, RW};

use proc_bitfield::bitfield;

pub const RDP_INTERFACE_BASE_ADDRESS: usize = 0xA4100000;

/// Registers of the RDP interface
///
#[repr(C)]
pub struct RDPInterface {

    /// Start address in RDRAM / DMEM for a DMA transfer of RDP primitives
    pub dp_start: RW<u32, u32>,

    /// End address in RDRAM / DMEM for a DMA transfer of RDP primitives (exclusive bound)
    pub dp_end: RW<u32, u32>,

    /// Current address in RDRAM / DMEM being transferred by the DMA engine
    pub dp_current: RO<u32>,

    /// Status and configuration of primitive processing and DMA transfer
    pub dp_status: RW<RDPStatusAsRead, RDPStatusAsWritten>,

}

impl RDPInterface {

    /// Returns memory-mapped RDP interface registers
    #[inline(always)]
    pub fn new() -> &'static mut Self {
        unsafe {
            &mut *(RDP_INTERFACE_BASE_ADDRESS as *mut Self)
        }
    }

}

bitfield! {
    /// RDP status register, as it's read
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct RDPStatusAsRead(pub u32): IntoRaw, FromRaw {
        pub start_pending: bool @ 10,
        pub end_pending: bool @ 9,
        pub busy: bool @ 6,
        pub flush: bool @ 2,
        pub freeze: bool @ 1,
        pub source: bool [DMATransferSource] @ 0,
    }
}

bitfield! {
    /// RDP status register, as it's written
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct RDPStatusAsWritten(pub u32): IntoRaw, FromRaw {
        pub reset_clock: bool @ 9,
        pub set_flush: bool @ 5,
        pub clear_flush: bool @ 4,
        pub set_freeze: bool @ 3,
        pub clear_freeze: bool @ 2,
        pub set_source_dmem: bool @ 1,
        pub set_source_xbus: bool @ 0,
    }
}

/// RDP commands are fed to the RDP from either of these sources.
pub enum DMATransferSource {

    /// RAM
    XBUS,

    /// RSP data memory
    DMEM,

}

impl From<bool> for DMATransferSource {
    fn from(value: bool) -> Self {
        match value {
            true => Self::DMEM,
            false => Self::XBUS,
        }
    }
}

impl Into<bool> for DMATransferSource {
    fn into(self) -> bool {
        match self {
            Self::XBUS => false,
            Self::DMEM => true,
        }
    }
}

// eof
