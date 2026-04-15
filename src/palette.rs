//! Color modes for rendering: solid, value gradient (iteration → palette),
//! and position gradient (two colors blended across the terminal window).
//!
//! Gradients are interpolated in linear-light space (gamma-decode →
//! interpolate → gamma-encode). Plain sRGB-space lerping darkens the middle
//! of transitions; linear interpolation keeps luminance stable.

use crate::fractal::Sample;

pub type Rgb = (u8, u8, u8);

/// Preset palettes for the value-gradient color mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Fire,
    Ocean,
    Grayscale,
    Rainbow,
    Electric,
}

impl Preset {
    pub const ALL: &'static [Preset] = &[
        Preset::Fire,
        Preset::Ocean,
        Preset::Grayscale,
        Preset::Rainbow,
        Preset::Electric,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Preset::Fire => "fire",
            Preset::Ocean => "ocean",
            Preset::Grayscale => "grayscale",
            Preset::Rainbow => "rainbow",
            Preset::Electric => "electric",
        }
    }

    pub fn cycle_next(self) -> Preset {
        let all = Self::ALL;
        let idx = all.iter().position(|&p| p == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn cycle_prev(self) -> Preset {
        let all = Self::ALL;
        let idx = all.iter().position(|&p| p == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }

    /// The ordered color stops for this preset, each paired with its
    /// position in `[0.0, 1.0]`.
    fn stops(self) -> &'static [(f32, Rgb)] {
        match self {
            Preset::Fire => &[
                (0.0, (0, 0, 0)),
                (0.3, (180, 20, 0)),
                (0.6, (240, 120, 0)),
                (0.85, (255, 220, 60)),
                (1.0, (255, 255, 230)),
            ],
            Preset::Ocean => &[
                (0.0, (0, 0, 15)),
                (0.3, (0, 30, 80)),
                (0.6, (20, 110, 180)),
                (0.85, (80, 200, 220)),
                (1.0, (230, 250, 255)),
            ],
            Preset::Grayscale => &[
                (0.0, (0, 0, 0)),
                (1.0, (255, 255, 255)),
            ],
            Preset::Rainbow => &[
                (0.0, (180, 0, 120)),
                (0.2, (200, 40, 40)),
                (0.4, (220, 180, 30)),
                (0.6, (60, 200, 80)),
                (0.8, (40, 120, 220)),
                (1.0, (130, 40, 200)),
            ],
            Preset::Electric => &[
                (0.0, (0, 0, 20)),
                (0.5, (120, 20, 220)),
                (1.0, (20, 240, 255)),
            ],
        }
    }
}

/// Axis along which a position gradient interpolates its two endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
    Diagonal,
}

impl Axis {
    pub const ALL: &'static [Axis] = &[Axis::Horizontal, Axis::Vertical, Axis::Diagonal];

    pub fn cycle_next(self) -> Axis {
        let all = Self::ALL;
        let idx = all.iter().position(|&a| a == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn name(self) -> &'static str {
        match self {
            Axis::Horizontal => "horizontal",
            Axis::Vertical => "vertical",
            Axis::Diagonal => "diagonal",
        }
    }
}

/// How a cell's color is computed.
#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    /// Single color, intensity from `Sample::t`.
    Solid(Rgb),
    /// Palette-sampled gradient driven by `Sample::t`.
    Value(Preset),
    /// Two colors blended across the window along `axis`, independent of t.
    Position { start: Rgb, end: Rgb, axis: Axis },
}

/// Pick a color for a single cell.
///
/// - `px`, `py` are the cell's column and row within a `w × h` grid.
/// - `sample` carries the fractal's escape result (density `t` and `class`).
///
/// Newton fractals return a non-zero `class`; we tint by basin and modulate
/// brightness by `t`, regardless of the active `ColorMode`.
pub fn pick_color(
    mode: &ColorMode,
    sample: Sample,
    px: u16,
    py: u16,
    w: u16,
    h: u16,
) -> Rgb {
    if sample.class != 0 {
        return basin_color(sample.class, sample.t);
    }
    match *mode {
        ColorMode::Solid(rgb) => scale_rgb(rgb, sample.t as f32),
        ColorMode::Value(preset) => sample_stops(preset.stops(), sample.t as f32),
        ColorMode::Position { start, end, axis } => {
            let t = position_factor(axis, px, py, w, h);
            lerp_rgb(start, end, t)
        }
    }
}

/// Basin color for Newton fractals. `class` is 1-indexed.
fn basin_color(class: u8, t: f64) -> Rgb {
    let base: Rgb = match class {
        1 => (240, 70, 90),   // root at 1 → warm red
        2 => (90, 220, 120),  // root at -0.5 + √3/2 i → green
        3 => (80, 140, 240),  // root at -0.5 - √3/2 i → blue
        _ => (200, 200, 200),
    };
    // Convergence speed fades toward black at boundary regions.
    scale_rgb(base, (0.25 + 0.75 * t as f32).clamp(0.0, 1.0))
}

fn position_factor(axis: Axis, px: u16, py: u16, w: u16, h: u16) -> f32 {
    let w = (w.max(1) - 1) as f32;
    let h = (h.max(1) - 1) as f32;
    match axis {
        Axis::Horizontal => (px as f32 / w).clamp(0.0, 1.0),
        Axis::Vertical => (py as f32 / h).clamp(0.0, 1.0),
        Axis::Diagonal => {
            let diag = (px as f32 / w + py as f32 / h) * 0.5;
            diag.clamp(0.0, 1.0)
        }
    }
}

fn sample_stops(stops: &[(f32, Rgb)], t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    if stops.len() < 2 {
        return stops.first().map(|s| s.1).unwrap_or((0, 0, 0));
    }
    // Find the bracketing pair of stops.
    for window in stops.windows(2) {
        let (ta, a) = window[0];
        let (tb, b) = window[1];
        if t <= tb {
            let local = if tb > ta { (t - ta) / (tb - ta) } else { 0.0 };
            return lerp_rgb(a, b, local);
        }
    }
    stops.last().unwrap().1
}

fn lerp_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    // Gamma-correct (linear-light) interpolation.
    let (ar, ag, ab) = linearize(a);
    let (br, bg, bb) = linearize(b);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    gamma_encode((lerp(ar, br), lerp(ag, bg), lerp(ab, bb)))
}

fn scale_rgb(c: Rgb, k: f32) -> Rgb {
    let k = k.clamp(0.0, 1.0);
    let (r, g, b) = linearize(c);
    gamma_encode((r * k, g * k, b * k))
}

fn linearize(c: Rgb) -> (f32, f32, f32) {
    (srgb_to_linear(c.0), srgb_to_linear(c.1), srgb_to_linear(c.2))
}

fn gamma_encode(lin: (f32, f32, f32)) -> Rgb {
    (
        linear_to_srgb(lin.0),
        linear_to_srgb(lin.1),
        linear_to_srgb(lin.2),
    )
}

fn srgb_to_linear(v: u8) -> f32 {
    let x = v as f32 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let x = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (x * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Rgb, b: Rgb, tol: i32) -> bool {
        (a.0 as i32 - b.0 as i32).abs() <= tol
            && (a.1 as i32 - b.1 as i32).abs() <= tol
            && (a.2 as i32 - b.2 as i32).abs() <= tol
    }

    #[test]
    fn position_horizontal_endpoints_reproduce() {
        let mode = ColorMode::Position {
            start: (0, 0, 255),
            end: (255, 0, 0),
            axis: Axis::Horizontal,
        };
        let s = Sample { t: 0.5, class: 0 };
        let left = pick_color(&mode, s, 0, 5, 100, 30);
        let right = pick_color(&mode, s, 99, 5, 100, 30);
        assert!(close(left, (0, 0, 255), 2), "left = {:?}", left);
        assert!(close(right, (255, 0, 0), 2), "right = {:?}", right);
    }

    #[test]
    fn position_vertical_endpoints_reproduce() {
        let mode = ColorMode::Position {
            start: (0, 0, 0),
            end: (255, 255, 255),
            axis: Axis::Vertical,
        };
        let s = Sample { t: 0.0, class: 0 };
        let top = pick_color(&mode, s, 50, 0, 100, 30);
        let bottom = pick_color(&mode, s, 50, 29, 100, 30);
        assert!(close(top, (0, 0, 0), 2));
        assert!(close(bottom, (255, 255, 255), 2));
    }

    #[test]
    fn value_gradient_endpoints_match_first_and_last_stop() {
        let mode = ColorMode::Value(Preset::Grayscale);
        let low = pick_color(&mode, Sample { t: 0.0, class: 0 }, 0, 0, 10, 10);
        let high = pick_color(&mode, Sample { t: 1.0, class: 0 }, 0, 0, 10, 10);
        assert!(close(low, (0, 0, 0), 2));
        assert!(close(high, (255, 255, 255), 2));
    }

    #[test]
    fn value_gradient_monotonic_luminance_for_grayscale() {
        let mode = ColorMode::Value(Preset::Grayscale);
        let mut prev = 0u16;
        for step in 0..=10 {
            let t = step as f64 / 10.0;
            let (r, g, b) = pick_color(&mode, Sample { t, class: 0 }, 0, 0, 10, 10);
            let lum = r as u16 + g as u16 + b as u16;
            assert!(lum >= prev, "luminance regressed at t={t}");
            prev = lum;
        }
    }

    #[test]
    fn newton_basin_overrides_color_mode() {
        let mode = ColorMode::Solid((200, 200, 200));
        let s = Sample { t: 1.0, class: 1 };
        let c = pick_color(&mode, s, 0, 0, 10, 10);
        // Basin 1 is red-dominant.
        assert!(c.0 > c.1 && c.0 > c.2, "expected red basin, got {:?}", c);
    }
}
