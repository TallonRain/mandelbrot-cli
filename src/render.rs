//! Parallel compute and buffered ANSI output for one frame.
//!
//! Compute: `rayon` per-row (good cache locality, each row is a contiguous
//! slice of the complex plane along the real axis).
//! Output: a single string with ANSI color escapes, written in one syscall.

use std::fmt::Write as _;
use std::io::{self, Write};

use rayon::prelude::*;

use crate::ascii;
use crate::fractal::Fractal;
use crate::palette::{self, ColorMode, Rgb};
use crate::viewport::Viewport;

/// One rendered cell: an ASCII glyph plus its foreground color.
#[derive(Debug, Clone, Copy)]
struct Cell {
    ch: char,
    color: Rgb,
}

/// Compute a frame and write it, with cursor positioning, to `out`.
///
/// The output sequence:
/// 1. `ESC [ H`   — home cursor (top-left of the terminal).
/// 2. For each row: cells, with `ESC [ 38;2;R;G;B m` prefixing every color
///    change. A `\r\n` separates rows (CR is needed in raw mode).
/// 3. `ESC [ 0 m` — reset attributes.
pub fn render<W: Write>(
    out: &mut W,
    fractal: &dyn Fractal,
    viewport: &Viewport,
    color_mode: &ColorMode,
    max_iter: u32,
    w: u16,
    h: u16,
) -> io::Result<()> {
    let rows = compute_rows(fractal, viewport, color_mode, max_iter, w, h);
    write_rows(out, &rows, w, h)
}

/// Compute a frame as a plain string with no color escapes. Used by the
/// `--oneshot` mode for CI and by snapshot tests.
pub fn render_plain(
    fractal: &dyn Fractal,
    viewport: &Viewport,
    max_iter: u32,
    w: u16,
    h: u16,
) -> String {
    let mut out = String::with_capacity(((w as usize) + 1) * h as usize);
    let rows: Vec<Vec<char>> = (0..h)
        .into_par_iter()
        .map(|row| {
            (0..w)
                .map(|col| {
                    let c = viewport.pixel_to_complex(col, row, w, h);
                    let s = fractal.sample(c, max_iter);
                    ascii::density_char(s.t)
                })
                .collect()
        })
        .collect();
    for (i, row) in rows.iter().enumerate() {
        for &ch in row {
            out.push(ch);
        }
        if i + 1 < rows.len() {
            out.push('\n');
        }
    }
    out
}

fn compute_rows(
    fractal: &dyn Fractal,
    viewport: &Viewport,
    color_mode: &ColorMode,
    max_iter: u32,
    w: u16,
    h: u16,
) -> Vec<Vec<Cell>> {
    (0..h)
        .into_par_iter()
        .map(|row| {
            (0..w)
                .map(|col| {
                    let c = viewport.pixel_to_complex(col, row, w, h);
                    let s = fractal.sample(c, max_iter);
                    let ch = ascii::density_char(s.t);
                    let color = palette::pick_color(color_mode, s, col, row, w, h);
                    Cell { ch, color }
                })
                .collect()
        })
        .collect()
}

fn write_rows<W: Write>(out: &mut W, rows: &[Vec<Cell>], w: u16, h: u16) -> io::Result<()> {
    // Estimate: ~20 bytes per cell in the worst case (full color escape).
    let mut buf = String::with_capacity((w as usize) * (h as usize) * 8);
    buf.push_str("\x1b[H");
    let mut last: Option<Rgb> = None;
    for row in rows {
        for cell in row {
            if Some(cell.color) != last {
                let _ = write!(
                    buf,
                    "\x1b[38;2;{};{};{}m",
                    cell.color.0, cell.color.1, cell.color.2
                );
                last = Some(cell.color);
            }
            buf.push(cell.ch);
        }
        buf.push_str("\r\n");
    }
    buf.push_str("\x1b[0m");
    out.write_all(buf.as_bytes())?;
    out.flush()
}
