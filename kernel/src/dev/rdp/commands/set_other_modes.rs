// SPDX-License-Identifier: GPL-3.0-or-later

//! RDP Command - Set Other Modes

use num_enum::{FromPrimitive, IntoPrimitive};
use proc_bitfield::bitfield;

bitfield! {

    /// General configuration of the RDP
    ///
    pub struct SetOtherModes(pub u64): FromRaw, IntoRaw {

        /// 0x2f
        pub opcode: u8 @ 56..=63,

        /// Complete drawing to the canvas at each command before proceeding to the next
        pub atomic_primitive_enable: bool @ 55,

        /// Sets the main rendering mode; see the associated enum.
        pub cycle_type: u8 [CycleType] @ 52..=53,

        /// Enable "perspective correction" for textures during texture sampling.
        ///
        /// See the introductory paragraph, here:
        /// - https://ultra64.ca/files/documentation/online-manuals/man/pro-man/pro13/index.html
        pub enable_perspective_correction_for_textures: bool @ 51,

        /// Enable the "detail texture" during texture sampling.
        ///
        /// See the "Detail Texture" section, here:
        /// - https://ultra64.ca/files/documentation/online-manuals/man/pro-man/pro13/13-07.html
        pub enable_detail_texture: bool @ 50,

        /// Enable sharpening during texture sampling.
        ///
        /// See the "Sharpen Mode" section, here:
        /// - https://ultra64.ca/files/documentation/online-manuals/man/pro-man/pro13/13-07.html
        pub enable_sharpened_texture: bool @ 49,

        /// Enable "Level of Detail" during texture sampling.
        ///
        /// See the "LOD Enabled" section, here:
        /// - https://ultra64.ca/files/documentation/online-manuals/man/pro-man/pro13/13-07.html
        pub enable_texture_level_of_detail: bool @ 48,

        /// Enable the texture lookup table ("TLUT")
        pub enable_texture_lookup_table: bool @ 47,

        /// Color model and corresponding pixel size of texels in the TLUT
        pub tlut_texel_type: bool [TexelTypeInTLUT] @ 46,

        /// Use either a single texel or group of texels during texture sampling
        pub texture_sample_method: bool [TextureSampleMethod] @ 45,

        /// Enable 2x2 texel interpolation during texel filtering. Official
        /// documentation notes this feature is "primarily used for MPEG (1)
        /// motion compensation processing".
        pub enable_half_texel_interpolation: bool @ 44,

        /// Enable bi­linear interpolation in cycle 0 when 1-Cycle or 2-Cycle mode is enabled
        pub enable_bilinear_interpolation_in_cycle_0: bool @ 43,

        /// Enable bi­linear interpolation in cycle 1 when 1-Cycle or 2-Cycle mode is enabled
        pub enable_bilinear_interpolation_in_cycle_1: bool @ 42,

        /// Color convert the texel outputted by the texture filter in cycle 0
        pub enable_color_convert_in_cycle_0: bool @ 41,

        /// Enable chroma keying"
        pub enable_chroma_keying: bool @ 40,

        /// Type of dithering done to RGB values in 1 Cycle or 2 Cycle modes
        pub rgb_dither_type: u8 [RGBDitherType] @ 38..=39,

        /// Type of dithering done to alpha values in 1 Cycle or 2 Cycle modes
        pub alpha_dither_type: u8 [AlphaDitherType] @ 36..=37,

        // Unknown or unused field; default observed to be 0xf
        // reserved_01: u8 @ 32..=35,

        /// Multiply blend 1a input in cycle 0
        pub mul_blend_1a_in_cycle_0: u8 @ 30..=31,

        /// Multiply blend 1a input in cycle 1
        pub mul_blend_1a_in_cycle_1: u8 @ 28..=29,

        /// Multiply blend 1b input in cycle 0
        pub mul_blend_1b_in_cycle_0: u8 @ 26..=27,

        /// Multiply blend 1b input in cycle 1
        pub mul_blend_1b_in_cycle_1: u8 @ 24..=25,

        /// Multiply blend 2a input in cycle 0
        pub mul_blend_2a_in_cycle_0: u8 @ 22..=23,

        /// Multiply blend 2a input in cycle 1
        pub mul_blend_2a_in_cycle_1: u8 @ 20..=21,

        /// Multiply blend 2b input in cycle 0
        pub mul_blend_2b_in_cycle_0: u8 @ 18..=19,

        /// Multiply blend 2b input in cycle 1
        pub mul_blend_2b_in_cycle_1: u8 @ 16..=17,

        /// Enable force blend
        pub force_blend: bool @ 14,

        /// Enable use of coverage bits in alpha calculation
        pub alpha_cvg_select: bool @ 13,

        /// Enable multiplying coverage bits by alpha value for final pixel alpha
        pub cvg_times_alpha: bool @ 12,

        /// Mode select for Z buffer
        pub z_mode: u8 [ZMode] @ 10..=11,

        /// Mode select for handling coverage values
        pub cvg_mode: u8 [CoverageMode] @ 8..=9,

        /// Only update color on coverage overflow
        pub update_color_on_cvg_overflow: bool @ 7,

        /// Enable coverage read/modify/write access to the canvas
        pub enable_rw_on_canvas_for_coverage: bool @ 6,

        /// Enable writing new Z value if color write is enabled
        pub z_update_en: bool @ 5,

        /// Enable conditional color write based on depth comparison
        pub z_compare_en: bool @ 4,

        /// Enable anti-aliasing based on coverage bits if force blend is not enabled
        pub enable_antialiasing_on_cvg: bool @ 3,

        /// Set the source of the Z value
        pub source_of_z: bool [ZSource] @ 2,

        /// Set the source for alpha comparisons
        pub alpha_compare_source: bool [AlphaCompareSource] @ 1,

        /// Enable conditional color write based on alpha compare
        pub alpha_compare_enable: bool @ 0,

    }

}

/// RDP render modes; this is not comprehensive documentation. The modes are
/// classified as either "standard" or "fast". In regular validated usage, the
/// standard modes are used for drawing triangles, whereas fast modes are used
/// for drawing rectangles.
///
/// See community notes, here:
/// https://discord.com/channels/205520502922543113/205522877343072266/1159561141610098848
///
/// Note the official programming documentation covers specific interactions of
/// these values with various commands (e.g. OneCycle w/ any rectangle command).
/// See the "Texture Mapping" section of that document in full.
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CycleType {

    /// A "standard" mode. Use 1 RDP cycle per pixel.
    #[default]
    OneCycle = 0b00,

    /// A "standard" mode. Use 2 RDP cycles per pixel; that is, two passes over
    /// various pipeline stages (e.g the texture, combiner and blender stages).
    /// Each pixel is slower to work through the pipeline, but not necessarily
    /// two times slower than OneCycle.
    TwoCycle = 0b01,

    /// A "fast" mode (writes 4 pixels at a time). Primarily used with the
    /// "textured rectangle" command, and can cause garbage output when used
    /// with the "fill rectangle" command.
    Copy = 0b10,

    /// A "fast" mode (writes 4 pixels at a time). Skips the texture pipeline.
    Fill = 0b11,

}

/// The RDP supports these color models and corresponding pixel sizes for texels
/// in the texture look-up table ("TLUT").
///
/// See the "Texture Image Types and Format" section, here:
/// - http://ultra64.ca/files/documentation/online-manuals/man/pro-man/pro12/12-04.html
pub enum TexelTypeInTLUT {

    /// 16-bit RGBA (5/5/5/1)
    RGBA16b,

    /// 16-bit "Intensity Alpha" (8/8)
    IA,

}

impl From<bool> for TexelTypeInTLUT {
    fn from(value: bool) -> Self {
        match value {
            true => Self::IA,
            false => Self::RGBA16b,
        }
    }
}

impl Into<bool> for TexelTypeInTLUT {
    fn into(self) -> bool {
        match self {
            Self::RGBA16b => false,
            Self::IA => true,
        }
    }
}

/// Available methods to sample textures
///
/// See these sections in the official programming manual:
///     - "Sampling Overview"
///     - "Bilinear Filtering and Point Sampling"
///
pub enum TextureSampleMethod {

    /// Sample from a single texel (1x1)
    Point,

    /// Sample from four texels (2x2)
    Bilinear,

}

impl From<bool> for TextureSampleMethod {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Point,
            true => Self::Bilinear,
        }
    }
}

impl Into<bool> for TextureSampleMethod {
    fn into(self) -> bool {
        match self {
            Self::Point => false,
            Self::Bilinear => true,
        }
    }
}

/// "Type of dithering done to RGB values in 1 Cycle or 2 Cycle modes"
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RGBDitherType {

    /// todo: doc
    MagicSquareMatrix = 0b00,

    /// todo: doc
    BayerMatrix = 0b01,

    /// todo: doc
    Noise = 0b10,

    /// todo: doc
    #[default]
    NoDither = 0b11,

}

/// "Type of dithering done to RGB values in 1 Cycle or 2 Cycle modes"
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AlphaDitherType {

    /// todo: doc
    Pattern = 0b00,

    /// todo: doc
    AntiPattern = 0b01,

    /// todo: doc
    Noise = 0b10,

    /// todo: doc
    #[default]
    NoDither = 0b11,

}


/// "Mode select for Z buffer"
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ZMode {

    /// todo: doc
    #[default]
    Opaque = 0b00,

    /// todo: doc
    Interpenetrating = 0b01,

    /// todo: doc
    Transparent = 0b10,

    /// todo: doc
    Decal = 0b11,

}

/// "Mode select for handling coverage values"
///
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CoverageMode {

    /// todo: doc
    #[default]
    Clamp = 0b00,

    /// todo: doc
    Wrap = 0b01,

    /// todo: doc
    ForceToFullCoverage = 0b10,

    /// todo: doc
    NoWriteback = 0b11,

}

/// Source of Z values
///
pub enum ZSource {

    /// todo: doc
    Pixel,

    /// todo: doc
    Primitive,

}

impl From<bool> for ZSource {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Pixel,
            true => Self::Primitive,
        }
    }
}

impl Into<bool> for ZSource {
    fn into(self) -> bool {
        match self {
            Self::Pixel => false,
            Self::Primitive => true,
        }
    }
}

/// Source for alpha comparisons
///
pub enum AlphaCompareSource {

    /// todo: doc
    Noise,

    /// todo: doc
    Blend,

}

impl From<bool> for AlphaCompareSource {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Noise,
            true => Self::Blend,
        }
    }
}

impl Into<bool> for AlphaCompareSource {
    fn into(self) -> bool {
        match self {
            Self::Noise => false,
            Self::Blend => true,
        }
    }
}

// eof
