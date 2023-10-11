// SPDX-License-Identifier: GPL-3.0-or-later

//! Raw definition of RDP commands (as bit fields, enums, and associated constants).
//!
//! The documentation linked below is not comprehensive. They are only used to
//! define the names, position, width, and a brief description of fields. This
//! collection of modules does not perform the necessary validation to use the
//! commands, although it may provide some supporting interfaces and tools.
//!
//! Documentation:
//!     - https://dragonminded.com/n64dev/Reality%20Coprocessor.pdf
//!     - https://n64brew.dev/wiki/Reality_Display_Processor/Commands
//!     - https://ultra64.ca/files/documentation/silicon-graphics/SGI_RDP_Command_Summary.pdf
//!     - https://ultra64.ca/files/documentation/nintendo/Nintendo_64_Programming_Manual_NU6-06-0030-001G_HQ.pdf
//!

pub mod fill_rectangle;
pub mod full_sync;
pub mod set_color_image;
pub mod set_fill_color;
pub mod set_other_modes;
pub mod set_scissor;

/// A simple list of RDP commands that have been defined by the modules above.
#[allow(non_camel_case_types)]
pub enum RDPCommands {
    FILL_RECTANGLE,
    FULL_SYNC,
    SET_COLOR_IMAGE,
    SET_FILL_COLOR,
    SET_OTHER_MODES,
    SET_SCISSOR,
}

impl RDPCommands {

    /// A mapping from RDP commands to their opcodes.
    #[inline(always)]
    pub const fn opcode(&self) -> u8 {
        match *self {
            Self::FILL_RECTANGLE => 0x36,
            Self::FULL_SYNC => 0x29,
            Self::SET_COLOR_IMAGE => 0x3F,
            Self::SET_FILL_COLOR => 0x37,
            Self::SET_OTHER_MODES => 0x2F,
            Self::SET_SCISSOR => 0x2D,
        }
    }

}

// eof
