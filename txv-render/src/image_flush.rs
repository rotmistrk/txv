//! Image rendering for terminal backends — Kitty and iTerm2 protocols.

use std::env;
use std::io::Write;

use txv_core::buffer::Buffer;
use txv_core::cell::Color;
use txv_core::image::ImagePlacement;

use crate::image_protocol::{CellPixelSize, ImageProtocol};

/// Check if we're inside tmux (need passthrough wrapping).
fn in_tmux() -> bool {
    env::var("TMUX").is_ok()
}

/// Render all image placements from a buffer to the terminal.
pub fn flush_images(out: &mut impl Write, buffer: &Buffer, protocol: ImageProtocol, cell_size: CellPixelSize) {
    if protocol == ImageProtocol::None {
        return;
    }
    let images = buffer.images();
    for img in images {
        match protocol {
            ImageProtocol::Kitty => emit_kitty(out, img, cell_size),
            ImageProtocol::Iterm2 => emit_iterm2(out, img, cell_size),
            ImageProtocol::None => {}
        }
    }
}

/// A visible rectangular run within an image placement.
#[allow(dead_code)]
struct VisibleRun {
    /// Cell column (relative to image rect origin).
    x: u16,
    /// Cell row (relative to image rect origin).
    y: u16,
    /// Width in cells.
    w: u16,
}

/// Compute visible cell runs for an image — cells where bg is Transparent.
fn visible_region(buffer: &Buffer, img: &ImagePlacement) -> Vec<VisibleRun> {
    let r = img.rect();
    let mut runs = Vec::new();
    for row in 0..r.h() {
        let by = r.y() + row;
        if by >= buffer.height() {
            break;
        }
        collect_row_runs(buffer, r.x(), r.w(), by, row, &mut runs);
    }
    runs
}

fn collect_row_runs(buffer: &Buffer, rx: u16, rw: u16, by: u16, row: u16, runs: &mut Vec<VisibleRun>) {
    let mut col = 0;
    while col < rw {
        let bx = rx + col;
        if bx >= buffer.width() {
            break;
        }
        if buffer.cell(bx, by).style().bg() == Color::Transparent {
            let start = col;
            col = scan_transparent_run(buffer, rx, rw, by, col);
            runs.push(VisibleRun {
                x: start,
                y: row,
                w: col - start,
            });
        } else {
            col += 1;
        }
    }
}

fn scan_transparent_run(buffer: &Buffer, rx: u16, rw: u16, by: u16, mut col: u16) -> u16 {
    while col < rw {
        let cx = rx + col;
        if cx >= buffer.width() || buffer.cell(cx, by).style().bg() != Color::Transparent {
            break;
        }
        col += 1;
    }
    col
}

/// Emit image via Kitty graphics protocol.
fn emit_kitty(out: &mut impl Write, img: &ImagePlacement, cell_size: CellPixelSize) {
    let r = img.rect();
    let data = img.data();
    let img_w = data.width();
    let img_h = data.height();
    let pixel_w = r.w() as u32 * cell_size.width() as u32;
    let pixel_h = r.h() as u32 * cell_size.height() as u32;

    // For simplicity, emit the full image at the placement position.
    // Kitty supports virtual placements; we use the direct transmission approach.
    // Position cursor at image origin.
    let _ = write!(out, "\x1b[{};{}H", r.y() + 1, r.x() + 1);

    // Encode full RGBA data, scaled to pixel dimensions
    let scaled = scale_image(data.pixels(), img_w, img_h, pixel_w, pixel_h);
    let encoded = base64_encode(&scaled);

    // Kitty: transmit in chunks of 4096 bytes
    let mut offset = 0;
    let total = encoded.len();
    while offset < total {
        let chunk_end = (offset + 4096).min(total);
        let chunk = &encoded[offset..chunk_end];
        let more = if chunk_end < total {
            1
        } else {
            0
        };
        if offset == 0 {
            let _ = write!(
                out,
                "\x1b_Ga=T,f=32,s={pixel_w},v={pixel_h},c={},r={},m={more};{chunk}\x1b\\",
                r.w(),
                r.h(),
            );
        } else {
            let _ = write!(out, "\x1b_Gm={more};{chunk}\x1b\\",);
        }
        offset = chunk_end;
    }
}

/// Emit image via iTerm2 inline image protocol.
fn emit_iterm2(out: &mut impl Write, img: &ImagePlacement, _cell_size: CellPixelSize) {
    let r = img.rect();
    let data = img.data();

    // Position cursor
    let _ = write!(out, "\x1b[{};{}H", r.y() + 1, r.x() + 1);

    // Encode source image directly as PNG (terminal handles scaling)
    let png = crate::png_encode::encode_png(data.width(), data.height(), data.pixels());
    let encoded = base64_encode(&png);

    // Wrap in tmux passthrough if needed
    let tmux = in_tmux();
    if tmux {
        let _ = write!(out, "\x1bPtmux;\x1b");
    }
    let _ = write!(
        out,
        "\x1b]1337;File=inline=1;size={};width={}cells;height={}cells:{}\x07",
        png.len(),
        r.w(),
        r.h(),
        encoded,
    );
    if tmux {
        let _ = write!(out, "\x1b\\");
    }
    let _ = out.flush();
}

/// Nearest-neighbor scale of RGBA image.
fn scale_image(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    if src_w == dst_w && src_h == dst_h {
        return src.to_vec();
    }
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    for y in 0..dst_h {
        let sy = (y * src_h / dst_h).min(src_h - 1);
        for x in 0..dst_w {
            let sx = (x * src_w / dst_w).min(src_w - 1);
            let si = ((sy * src_w + sx) * 4) as usize;
            let di = ((y * dst_w + x) * 4) as usize;
            dst[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }
    dst
}

/// Simple base64 encoding (no dependency needed).
fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write as FmtWrite;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    let _ = out.write_str(""); // satisfy FmtWrite import
    out
}
