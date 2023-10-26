// SPDX-License-Identifier: GPL-3.0-or-later

//! Core the game
//!
//! This is currently just a simple visual demo, and an incomplete one at that.
//! There are many todo items...
//!

#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::dev::rdp::interface::RDPInterface;
use kernel::dev::rdp::commands as rdp_commands;
use kernel::dev::vi;
use kernel::pic::RGBA;

/// Addresses of two 640x480 32-bit RGBA frame buffers
///
/// With double buffering, two operations are occurring at any time:
///     - Reading, from a frame buffer by the video interface
///     - Writing, to another frame buffer by the RCP
///
/// If both frame buffers are placed on the same physical memory chip, these
/// writes and reads will conflict and cause increased memory latency. For the
/// resolution and bit depth chosen, this is impossible to avoid within 4MB (4x
/// 1MB chips). To mitigate, each buffer is placed so that the tail end of one
/// and the tip of the other extend into the 3rd MB.
///
/// See: https://www.youtube.com/watch?v=jbr-EwCqSfs&t=915s
///
const FRAME_BUFFER_1_VADDR: usize = 0xA0100000;  // ..0xA022C000
const FRAME_BUFFER_2_VADDR: usize = 0xA02D4000;  // ..0xA0400000

static mut DISPLAY_LIST: [u64; 73] = [0; 73];

/// Initializes the video interface (NTSC, 640x480 (480i), 32-bit color)
///
/// Documentation:
///     - https://n64brew.dev/wiki/Video_Interface
///
#[inline(never)]
fn init_vi() {
    let video_interface = vi::VI::new();
    let mut video_control = vi::VI_CTRL(0)
        .with_color_depth(vi::ColorDepth::Blank)  // stop the signal for setup
        .with_aa_mode(vi::AntiAliasMode::Disabled)
        .with_pixel_advance(0b11)
        .with_enable_serrate(true)
        .with_enable_dither_filter(false)
        .with_enable_divot(false)
        .with_enable_gamma_boost(false)
        .with_enable_gamma_dither(false);
    unsafe {
        video_interface.ctrl.write(video_control);
        video_interface.origin.write(
            vi::VI_ORIGIN(0)
                .with_vaddr(FRAME_BUFFER_1_VADDR as u32)
        );
        video_interface.width.write(
            vi::VI_WIDTH(0)
                .with_width(640 * 2)
        );
        video_interface.v_intr.write(
            vi::VI_V_INTR(0)
                .with_half_line(0x3ff)
        );
        video_interface.burst.write(
            vi::VI_BURST(0)
                .with_color_burst_start(62)
                .with_color_burst_width(34)
                .with_hsync_width(57)
                .with_vsync_width(5)
        );
        video_interface.v_sync.write(
            vi::VI_V_SYNC(0)
                .with_v_sync(524)
        );
        video_interface.h_sync.write(
            vi::VI_H_SYNC(0)
                .with_leap_pattern(0)
                .with_line_duration(3093)
        );
        video_interface.h_sync_leap.write(
            vi::VI_H_SYNC_LEAP(0)
                .with_leap_a(3093)
                .with_leap_b(3093)
        );
        video_interface.h_video.write(
            vi::VI_H_VIDEO(0)
                .with_h_start(108)
                .with_h_end(748)
        );
        video_interface.v_video.write(
            vi::VI_V_VIDEO(0)
                .with_v_start(37)
                .with_v_end(511)
        );
        video_interface.v_burst.write(
            vi::VI_V_BURST(0)
                .with_v_burst_start(0x00E)
                .with_v_burst_end(0x204)
        );
        video_interface.x_scale.write(
            vi::VI_X_SCALE(0)
                .with_offset(0)
                .with_scale(0b_01_0000000000)  // 2.10 fixed-point
        );
        video_interface.y_scale.write(
            vi::VI_Y_SCALE(0)
                .with_offset(0)
                .with_scale(0b_01_0000000000)  // 2.10 fixed-point
        );
        video_control.set_color_depth(vi::ColorDepth::TrueColor);  // begin the signal after setup
        video_interface.ctrl.write(video_control);
    }
}

/// Blank the frame buffers ("fb1" & "fb2") and write a test pattern.
///
#[inline(never)]
fn init_fbs() {

    // RDP display list to draw color bars to the frame buffers.
    let clear_fbs_display_list = &[

        rdp_commands::set_other_modes::SetOtherModes(0)
            .with_opcode(rdp_commands::RDPCommands::SET_OTHER_MODES.opcode())
            .with_atomic_primitive_enable(true)
            .with_cycle_type(rdp_commands::set_other_modes::CycleType::Fill)
            .into(),

        rdp_commands::set_scissor::SetScissor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_SCISSOR.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left(0)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        // Black the two frame buffers with a near-black color just light enough to
        // denote the projected frame against an emulator's default background or
        // other border.

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x01010100)
            .into(),

        rdp_commands::set_color_image::SetColorImage(0)
            .with_opcode(rdp_commands::RDPCommands::SET_COLOR_IMAGE.opcode())
            .with_address(FRAME_BUFFER_1_VADDR as u32)
            .with_model(rdp_commands::set_color_image::CanvasColorModel::RGBA)
            .with_pixel_size(rdp_commands::set_color_image::CanvasPixelSize::WORD)
            .with_width(640 - 1)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left(0)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_color_image::SetColorImage(0)
            .with_opcode(rdp_commands::RDPCommands::SET_COLOR_IMAGE.opcode())
            .with_address(FRAME_BUFFER_2_VADDR as u32)
            .with_model(rdp_commands::set_color_image::CanvasColorModel::RGBA)
            .with_pixel_size(rdp_commands::set_color_image::CanvasPixelSize::WORD)
            .with_width(640 - 1)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left(0)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        // Vertical color bars of bright primary colors

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFFFF00)
            .into(),

        // note: clipped out
        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 0) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 0)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x7F7F7F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 1) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 1)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF000000)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 2) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 2)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FF0000)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 3) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 3)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x0000FF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 4) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 4)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFF0000)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 5) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 5)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FFFF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 6) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 6)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF00FF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((16 * 7) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((7 + (16 * 7)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        // Vertical bars fo dim primary colors

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFFFF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 0))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 0)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x7F7F7F00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 1))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 1)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF000000 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 2))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 2)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FF0000 & 0x0F0F0F00)
            .into(),

            rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 3))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 3)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x0000FF00 & 0x0F0F0F00)
            .into(),

            rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 4))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 4)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFF0000 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 5))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 5)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FFFF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 6))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 6)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF00FF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left((639 - (7 + (16 * 7))) << 2)
            .with_y_upper_left(0)
            .with_x_lower_right((639 - (16 * 7)) << 2)
            .with_y_lower_right(479 << 2)
            .into(),

        // Horizontal bars of bright primary colors

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFFFF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 0) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 0)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x7F7F7F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 1) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 1)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF000000)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 2) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 2)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FF0000)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 3) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 3)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x0000FF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 4) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 4)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFF0000)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 5) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 5)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FFFF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 6) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 6)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF00FF00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((16 * 7) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((7 + (16 * 7)) << 2)
            .into(),

        // Horizontal bars of dim colors

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFFFF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 0))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 0)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x7F7F7F00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 1))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 1)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF000000 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 2))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 2)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FF0000 & 0x0F0F0F00)
            .into(),

            rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 3))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 3)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x0000FF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 4))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 4)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFFFF0000 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 5))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 5)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0x00FFFF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 6))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 6)) << 2)
            .into(),

        rdp_commands::set_fill_color::SetFillColor(0)
            .with_opcode(rdp_commands::RDPCommands::SET_FILL_COLOR.opcode())
            .with_packed_color(0xFF00FF00 & 0x0F0F0F00)
            .into(),

        rdp_commands::fill_rectangle::FillRectangle(0)
            .with_opcode(rdp_commands::RDPCommands::FILL_RECTANGLE.opcode())
            .with_x_upper_left(0)
            .with_y_upper_left((479 - (8 + (16 * 7))) << 2)
            .with_x_lower_right(639 << 2)
            .with_y_lower_right((479 - (16 * 7)) << 2)
            .into(),

        rdp_commands::full_sync::FullSync(0)
            .with_opcode(rdp_commands::RDPCommands::FULL_SYNC.opcode())
            .into(),

        0,

    ];

    // The display list sent to the RDP is formed by copying elements from the
    // slice above into a statically defined array in module-scope. This works
    // around a problem where arrays allocated in the stack frame do not write
    // their values to memory if the compiler can't observe use of the values.
    //
    // There are workarounds like the code below, but the long term follow-up
    // here is really to just get a heap allocator and never pass pointers from
    // the stack. So, this workaround will go away very soon.
    //
    unsafe {
        DISPLAY_LIST.clone_from_slice(clear_fbs_display_list)
    }

    // Submit the commands to the RDP; see the wiki for details.
    let rdpi = RDPInterface::new();
    let ptrs = clear_fbs_display_list.as_ptr_range();
    unsafe {
        rdpi.dp_start.write(ptrs.start as u32);
        rdpi.dp_end.write(ptrs.end as u32)
    }

    // Wait for the RDP to draw the bars
    loop {
        let status = rdpi.dp_status.read();
        if !status.command_busy() {
            break;
        }
    }

    draw_interlace_pattern();

}

#[inline(never)]
fn draw_interlace_pattern() {

    let fb2 = FRAME_BUFFER_2_VADDR as *mut u32;

    // Draw special vertical bars
    for col in (320 - 32)..(320 + 32) as isize {
        for row in 0..480 as isize {
            let offset = col + (640 * row);

            // Black/White alternation, the most pathological interlacing case.
            // Jitter on these line is expected if interlacing is implemented
            // and no deinterlacer is involved.
            if col < 320 {

                // Even/Odd field alternation
                if col < 320 - 16 {
                    unsafe {
                        if row & 0x1 == 0 {
                            *fb2.offset(offset) = 0xFFFFFF00;
                        } else {
                            *fb2.offset(offset) = 0x00000000;
                        }
                    }
                } else {
                    unsafe {
                        if row & 0x1 == 0 {
                            *fb2.offset(offset) = 0x00000000;
                        } else {
                            *fb2.offset(offset) = 0xFFFFFF00;
                        }
                    }
                }

            } else {

                let gradient_offset: u8 = (0xFF as u8).saturating_sub((row >> 1) as u8);

                // Produce a gradient
                let gradient_color: u32 = RGBA(0)
                    .with_red(gradient_offset)
                    .with_green(gradient_offset)
                    .with_blue(gradient_offset)
                    .with_alpha(0)
                    .into();

                // .. ditto ^, but with a gradient.
                // Demonstrates that closer hues shouldn't flicker, much.
                if col < 320 + 16 {
                    unsafe {
                        if row & 0x1 == 0 {
                            *fb2.offset(offset) = gradient_color;
                        } else {
                            *fb2.offset(offset) = 0xFFFFFF00;
                        }
                    }
                } else {
                    unsafe {
                        if row & 0x1 == 0 {
                            *fb2.offset(offset) = 0xFFFFFF00;
                        } else {
                            *fb2.offset(offset) = gradient_color;
                        }
                    }
                }

            }
        }
    }

    // Draw special horizontal bars. No expected vertical or horizontal flicker.
    for row in (240 - 32)..(240 + 32) as isize {
        for col in 0..640 as isize {
            let offset = col + (640 * row);
            if row < 240 {
                unsafe {
                    if col & 0x1 == 0 {
                        *fb2.offset(offset) = 0x33FF3300;
                    } else {
                        *fb2.offset(offset) = 0x33333300;
                    }
                }
            } else {
                unsafe {
                    if col & 0x1 == 0 {
                        *fb2.offset(offset) = 0xCCCCCC00;
                    } else {
                        *fb2.offset(offset) = 0xFF33FF00;
                    }
                }
            }
        }
    }

    // Draw special screen center. Expect flicker without deinterlacer at box top.
    let row_start = 240 - 32;
    let row_end = 240 + 32;
    let col_start = 320 - 32;
    let col_end = 320 + 32;
    for row in row_start..row_end as isize {
        for col in col_start..col_end as isize {
            let offset = col + (640 * row);
            let r: u8 = ((row_end - row) * 4) as u8;
            let g: u8 = (((row_end - row) + (col_end - col)) * 2) as u8;
            let b: u8 = ((col_end - col) * 4) as u8;
            let color = RGBA(0)
                .with_red(r)
                .with_green(g)
                .with_blue(b)
                .with_alpha(0);
            unsafe {
                *fb2.offset(offset) = color.into();
            }
        }
    }

}

#[no_mangle]
pub extern "C" fn __start() -> ! {
    init_vi();
    init_fbs();

    // No interrupt handler yet, so interlacing is done by polling the
    // current projected line from the VI and adjusting the frame buffer address
    // when it changes from the even to odd field.
    let video_interface = vi::VI::new();
    let mut offset: usize = 0;
    let mut swap: bool;
    let mut swap_counter = 0;
    unsafe {
        video_interface.origin.write(
            vi::VI_ORIGIN(0)
                .with_vaddr(FRAME_BUFFER_2_VADDR as u32)
        );
    }
    loop {
        let current_half_line: u16 = video_interface.v_current.read().half_line();

        // Only swap the projected field at the end of a frame
        // if current_half_line < 500 {
        //     continue;
        // }

        // If on an even half-line (an even field), adjust so odd half lines are
        // used on the next field (and v.v.).
        if current_half_line & 0x1 == 0 {
            swap = offset == 0;
            offset = 640;  // skip a line of 4-byte pixels
        } else {
            swap = offset > 0;
            offset = 0;
        }

        // Offset the address of the frame buffer so that the VI skips even or odd fields.
        if swap {
            unsafe {
                video_interface.origin.write(
                    vi::VI_ORIGIN(0)
                        .with_vaddr((FRAME_BUFFER_2_VADDR + (offset * 4)) as u32)
                );
            }
            swap_counter += 1;
            if swap_counter == 2 {
                draw_interlace_pattern();
            }
        }

    }

}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// eof
