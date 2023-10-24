// SPDX-License-Identifier: GPL-3.0-or-later

//! The video interface ("VI") of the N64 generates a composite video signal
//! using pixel data stored in a frame buffer and values given to associated
//! hardware registers. This module defines the register-based interface with
//! their value's associated bitfield and default value for a given video signal
//! standard (e.g. NTSC).
//!
//! Documentation:
//!     - https://n64brew.dev/wiki/Video_Interface
//!     - https://dragonminded.com/n64dev/Reality%20Coprocessor.pdf
//!     - https://en64.shoutwiki.com/wiki/VI_Registers_Detailed
//!

// TODO:
// - fixed-point format macros and types

#![allow(non_camel_case_types)]

use crate::dev::reg::RW;

use num_enum::{FromPrimitive, IntoPrimitive};
use proc_bitfield::bitfield;

pub const VIDEO_INTERFACE_BASE_ADDRESS: usize = 0xA4400000;

/// VI registers and associated bitfields
///
#[repr(C)]
pub struct VI {

    /// Miscellaneous features and configuration options for the VI, mostly
    /// affecting picture qualities independent of the target video signal
    /// standard (i.e. NTSC, PAL).
    pub ctrl: RW<VI_CTRL, VI_CTRL>,

    /// Virtual address in the KSEG1 segment of RDRAM pointing to the frame
    /// buffer from which the video interface should read pixels, which can
    /// changed at any time (e.g. for double buffering or interlacing).
    pub origin: RW<VI_ORIGIN, VI_ORIGIN>,

    /// Width in pixels of the frame buffer
    pub width: RW<VI_WIDTH, VI_WIDTH>,

    /// When the VI reaches this half line, a VI interrupt is generated.
    /// Default value is 0x3FF / 1023, but "usually set to the last line
    /// containing pixel data".
    pub v_intr: RW<VI_V_INTR, VI_V_INTR>,

    /// "The current half line, sampled once per line"
    /// Any written value will clear the current VI interrupt.
    pub v_current: RW<VI_V_CURRENT, u32>,

    /// Control over timing of video signal sections (e.g. color burst). In
    /// almost all cases, use constants defined by the video signal standard
    /// (i.e. NTSC or PAL).
    pub burst: RW<VI_BURST, VI_BURST>,

    /// "One less than the total number of visible and non-visible half-lines."
    pub v_sync: RW<VI_V_SYNC, VI_V_SYNC>,

    /// Horizontal scanline length and "leap" setting.
    pub h_sync: RW<VI_H_SYNC, VI_H_SYNC>,

    /// Horizontal scanline length during vsync, mostly useful for non-NTSC modes
    pub h_sync_leap: RW<VI_H_SYNC_LEAP, VI_H_SYNC_LEAP>,

    /// In pixels, this is the start and end of the "active video image".
    pub h_video: RW<VI_H_VIDEO, VI_H_VIDEO>,

    /// In half-lines, this is the start and end of the "active video image".
    pub v_video: RW<VI_V_VIDEO, VI_V_VIDEO>,

    /// In half-lines, this is the start and end of the "color burst enable".
    pub v_burst: RW<VI_V_BURST, VI_V_BURST>,

    /// Frame buffer horizontal scale factor in 2.10 fixed-point format.
    pub x_scale: RW<VI_X_SCALE, VI_X_SCALE>,

    /// Frame buffer vertical scale factor in 2.10 fixed-point format.
    pub y_scale: RW<VI_Y_SCALE, VI_Y_SCALE>,

}

impl VI {

    /// Returns memory-mapped video interface registers
    ///
    #[inline(always)]
    pub fn new() -> &'static mut Self {
        unsafe {
            &mut *(VIDEO_INTERFACE_BASE_ADDRESS as *mut Self)
        }
    }

}

bitfield! {

    /// Miscellaneous features and configuration options for the VI
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_CTRL(pub u32): IntoRaw, FromRaw {

        /// Enable the VI to de-dither the bitmap. Normally used with 16-bit
        /// color where the RDP has applied its "Magic square matrix" dither
        /// type and combined with antialiasing to reduce vertical banding.
        pub enable_dither_filter: bool @ 16,

        /// Unknown; requires observed default value of 0b11.
        pub pixel_advance: u8 @ 12..=15,

        /// How the VI should interpolate pixels
        pub aa_mode: u8 [AntiAliasMode] @ 8..=9,

        /// Presumably enables the short pulses ("serrations") within the vsync,
        /// which are defined by the NTSC standard. Interlace scanning depends
        /// on the presence and timing of these pulses. Serration in the vsync
        /// seems optional for progressive scan, and in the cases where an N64
        /// uses progressive scanning this setting should be disabled.
        ///
        /// Documentation:
        /// - https://ultra64.ca/files/documentation/online-manuals/functions_reference_manual_2.0i/os/osVi.html
        /// - https://electronics.stackexchange.com/a/598889
        /// - https://people.ece.cornell.edu/land/courses/ece5760/video/gvworks/GV%27s%20works%20%20NTSC%20demystified%20-%20Cheats%20-%20Part%206.htm
        pub enable_serrate: bool @ 6,

        /// Enables the "divot circuit", which reduces some artifacts from AA
        ///
        /// Documentation:
        /// - https://ultra64.ca/files/documentation/online-manuals/man/pro-man/pro15/15-07.html
        pub enable_divot: bool @ 4,

        /// Enables gamma correction
        pub enable_gamma_boost: bool @ 3,

        /// Enables gamma-aware dithering?
        /// https://www.nayuki.io/page/gamma-aware-image-dithering
        pub enable_gamma_dither: bool @ 2,

        /// Color depth of the frame buffer (+ video signal toggle?)
        pub color_depth: u8 [ColorDepth] @ 0..=1,

    }

}

/// Anti-aliasing and resampling control
///
/// Notes:
///     - AA must be enabled with 240p or the video image will be corrupt.
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AntiAliasMode {

    /// Enable anti-aliasing, resampling, and always fetch extra "lines"
    Always = 0b00,

    /// Enable anti-aliasing, resampling, and only fetch extra "lines" as needed
    Enabled = 0b01,

    /// Disable anti-aliasing, enable resampling
    ResampleOnly = 0b10,

    /// No AA or resampling; replicate pixels without interpolation
    #[default]
    Disabled = 0b11,

}

/// Color depth of the frame buffer or blank screen
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ColorDepth {

    /// No signal?
    #[default]
    Blank = 0b00,

    /// Frame buffer uses 16-bits per pixel, 5/5/5/1 RGBA
    HighColor = 0b10,

    /// Frame buffer uses 32-bits per pixel, 8/8/8/8 RGBA
    TrueColor = 0b11,

}

bitfield! {

    /// Virtual address in the RDRAM KSEG1 segment pointing to the frame buffer
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_ORIGIN(pub u32): IntoRaw, FromRaw {
        pub vaddr: u32 @ 0..=23,
    }

}

bitfield! {

    /// Width of the frame buffer in pixels
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_WIDTH(pub u32): IntoRaw, FromRaw {
        pub width: u16 @ 0..=11,
    }

}

bitfield! {

    /// The integer-valued half-line at which the VI interrupt is triggered.
    /// "Usually set to the last line containing pixel data."
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_V_INTR(pub u32): IntoRaw, FromRaw {
        pub half_line: u16 @ 0..=9,
    }

}

bitfield! {

    /// Current integer-valued half-line being projected by the VI. When
    /// interlacing, the last bit reflects the field number ("even" or "odd").
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_V_CURRENT(pub u32): IntoRaw, FromRaw {
        pub half_line: u16 @ 0..=9,
    }

}

bitfield! {

    /// Timing details (in terms of pixels) for sections of the video signal.
    ///
    /// Usually set from constants derived from time values defined by the video
    /// signal standard (i.e. NTSC or PAL) and the VI's internal clock rate.
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_BURST(pub u32): IntoRaw, FromRaw {

        /// Timing of the start of color burst in terms of pixels.
        ///
        /// Standard values from the N64Brew wiki:
        /// - NTSC: 62
        /// -  PAL: 64
        pub color_burst_start: u16 @ 20..=29,

        /// Timing of the length of vsync in terms of half-lines.
        ///
        /// Standard values from the N64Brew wiki:
        /// - NTSC: 5
        /// -  PAL: 4
        pub vsync_width: u8 @ 16..=19,

        /// Timing of the length of color burst in terms of pixels.
        ///
        /// Standard values from the N64Brew wiki:
        /// - NTSC: 34
        /// -  PAL: 35
        pub color_burst_width: u8 @ 8..=15,

        /// Timing of the length of hsync in terms of pixels.
        ///
        /// Standard values from the N64Brew wiki:
        /// - NTSC: 57
        /// -  PAL: 58
        pub hsync_width: u8 @ 0..=7,

    }

}

bitfield! {

    /// Integer-valued number of half-lines per "field". Usually set to a
    /// constant derived from the video signal standard (i.e. NTSC or PAL)
    /// and VI internal clock rate.
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_V_SYNC(pub u32): IntoRaw, FromRaw {

        /// Integer-valued number of half-lines per "field".
        ///
        /// "This should match either NTSC/MPAL (non-interlaced: 525, interlaced: 524)
        /// or PAL (non-interlaced: 625, interlaced: 624)"
        pub v_sync: u16 @ 0..=9,

    }

}

bitfield! {

    /// Timing detail for horizontal lines (in terms of quarter pixels).
    ///
    /// Usually set from constants derived from time values defined by the video
    /// signal standard (i.e. NTSC or PAL) and the VI's internal clock rate.
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_H_SYNC(pub u32): IntoRaw, FromRaw {

        /// "1-per-vsync selector as whether to use LEAP_A or LEAP_B, with a
        /// repeating pattern of every 5 vsyncs". - @lidnariq
        ///
        /// For PAL, use constant 0x15. For NTSC, so long as the leap values
        /// (VI_H_SYNC_LEAP) are equal, the leap pattern does not matter.
        ///
        /// Notes:
        /// - https://discord.com/channels/205520502922543113/768169699564453910/1125620255927050422
        pub leap_pattern: u8 @ 16..=20,

        /// Duration (in terms of quarter pixels) of any horizontal line.
        ///
        /// "One less than the total length of a scanline in 1/4 pixel units.
        /// Should always use standard values: NTSC (3093), PAL (3177), or MPAL
        /// (3090) Default value of 0x7FF"
        pub line_duration: u16 @ 0..=11,

    }

}

bitfield! {

    /// Timing of a horizontal line during vsync per field in quarter pixels
    /// (see the link). For NTSC, set both leap values to the line duration
    /// given by VI_H_SYNC.
    ///
    /// Summarized from the wiki:
    ///
    /// - Supports PAL's "one extra chroma period per 625 whole scanlines emitted"
    ///
    /// - These are associated with a counter that initiates at onset of vsync.
    ///   When it's equal to the given value for the current field, the VI starts
    ///   or restarts the a scanline of vsync.
    ///
    /// - Which scanline of vsync is affected depends on how the value compares
    ///   to H_SYNC (larger: second scanline of vsync, otherwise the first).
    ///   Smaller values are noted to cause unintended effects, like complete
    ///   omission of one hsync or keeping the csync (physical pin) mistakenly
    ///   high for an entire scanline. Enabling serration causes other effects.
    ///
    /// Discussion:
    /// - https://discord.com/channels/205520502922543113/768169699564453910/1123508258213204078
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_H_SYNC_LEAP(pub u32): IntoRaw, FromRaw {

        /// On even fields, the horizontal line length during vsync in quarter pixels
        pub leap_a: u16 @ 16..=27,

        /// On odd fields, the horizontal line length during vsync in quarter pixels
        pub leap_b: u16 @ 0..=11,

    }

}

bitfield! {

    /// Start and end timing of horizontal "video" in terms of pixels, derived
    /// from the signal standard and the VI's internal clock rate.
    ///
    /// "The difference between these values is normally 640 pixels."
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_H_VIDEO(pub u32): IntoRaw, FromRaw {

        /// Timing of the start of horizontal "video" in terms of pixels.
        /// "Typical values: NTSC (108) or PAL (128)"
        pub h_start: u16 @ 16..=25,

        /// Timing of the end of horizontal "video" in terms of pixels.
        /// "Typical values: NTSC (748) or PAL (768)"
        pub h_end: u16 @ 0..=9,

    }

}

bitfield! {

    /// Start and end timing of vertical "video" in terms of half-lines, derived
    /// from the signal standard and the VI's internal clock rate.
    ///
    /// "The difference between these values is normally 474 lines."
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_V_VIDEO(pub u32): IntoRaw, FromRaw {

        /// Timing of the start of vertical "video" in terms of half-lines.
        /// "Typical values: NTSC (0x025) or PAL (0x05F)"
        pub v_start: u16 @ 16..=25,

        /// Timing of the end of vertical "video" in terms of half-lines.
        /// "Typical values: NTSC (0x1FF) or PAL (0x239)"
        pub v_end: u16 @ 0..=9,

    }

}

bitfield! {

    /// Start and end timing in terms of half-lines of the "color burst"
    /// section of the video signal.
    ///
    /// Note a thought about disabling the color burst:
    ///
    /// - "As it turns out, on the earlier N64s, colorburst is also the "clamp"
    ///   signal, which lets the RGB-to-composite encoder know where black is.
    ///   Without this, after a minute the entire video signal collapses to 0V. So
    ///   you still have to assert colorburst, but you just have to assert it at
    ///   the wrong time. The upper blanking region would be good. Something that
    ///   I would like other people to test is what happens on N64s with the later
    ///   video DACs." - @lidnariq
    ///
    ///   https://discord.com/channels/205520502922543113/205522877343072266/1166144673593692220
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_V_BURST(pub u32): IntoRaw, FromRaw {

        /// Timing of the start of the color burst in terms of half-lines.
        /// "Typical values: NTSC (0x00E) or PAL (0x009)"
        pub v_burst_start: u16 @ 16..=25,

        /// Timing of the end of the color burst in terms of half-lines.
        /// "Typical values: NTSC (0x204) or PAL (0x26B)"
        pub v_burst_end: u16 @ 0..=9,

    }

}

bitfield! {

    /// Horizontal offset and scaling
    ///
    /// Notes from the N64Brew wiki:
    ///
    /// - "If AA_MODE = 11 (resampling disabled), TYPE = 10 (16-bit), X_SCALE is
    ///   0x200 or lower, and H_START is less than 128, the VI generates invalid
    ///   output, consisting of the first 64 pixels from the frame buffer from
    ///   the current line, then 64 pixels of garbage, and these two repeat for
    ///   the rest of each scanline"
    ///
    /// - "If X_SCALE is higher than 0x800 (32bpp) or 0xE00 (16bpp), the scaler
    ///   renders incorrect pixels, with specifics depending on depth. This
    ///   appears to be due to exceeding the number of VI fetches allocated per
    ///   scanline."
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_X_SCALE(pub u32): IntoRaw, FromRaw {

        /// Sub-pixels offset in 2.10 fixed-point format
        pub offset: u16 @ 16..=27,

        /// Reciprocal scale-up factor in 2.10 fixed-point format
        ///
        /// "Without any blending, the scaling factor specified is the number of
        /// source pixels to advance per output pixel emitted" - @lidnariq
        pub scale: u16 @ 0..=11,

    }

}

bitfield! {

    /// Vertical offset and scaling
    ///
    /// Notes from the N64Brew wiki:
    ///
    /// - "If Y_SCALE exceeds 0xC00, it instead behaves like a glitchy
    ///    variation of 3*(0x1000-Y_SCALE)"
    ///
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct VI_Y_SCALE(pub u32): IntoRaw, FromRaw {

        /// Sub-pixels offset in 2.10 fixed-point format
        pub offset: u16 @ 16..=27,

        /// Reciprocal scale-up factor in 2.10 fixed-point format
        ///
        /// "Without any blending, the scaling factor specified is the number of
        /// source pixels to advance per output pixel emitted" - @lidnariq
        pub scale: u16 @ 0..=11,

    }

}

// eof
