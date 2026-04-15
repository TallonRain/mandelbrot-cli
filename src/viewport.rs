//! View onto the complex plane. Maps (column, row) → Complex<f64> with
//! aspect correction for non-square character cells.

use num_complex::Complex;

/// Terminal character cells are roughly twice as tall as they are wide.
/// We apply this ratio to the vertical step so circles stay round.
const CELL_ASPECT: f64 = 2.0;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Point in the complex plane shown at the screen's center.
    pub center: Complex<f64>,
    /// Horizontal half-width of the visible region, in complex-plane units.
    pub scale: f64,
}

impl Viewport {
    /// Default view centered on the classic Mandelbrot frame (x ∈ [-2, 1]).
    pub fn default_mandelbrot() -> Self {
        Self {
            center: Complex::new(-0.5, 0.0),
            scale: 1.5,
        }
    }

    /// Convert a pixel (column, row) within a `w × h` grid into a point on
    /// the complex plane. The returned point is the center of the cell.
    pub fn pixel_to_complex(&self, col: u16, row: u16, w: u16, h: u16) -> Complex<f64> {
        let w = w.max(1) as f64;
        let h = h.max(1) as f64;
        let step_x = 2.0 * self.scale / w;
        let step_y = step_x * CELL_ASPECT;
        let dx = col as f64 + 0.5 - w / 2.0;
        let dy = row as f64 + 0.5 - h / 2.0;
        Complex::new(
            self.center.re + dx * step_x,
            // Screen row grows downward; the imaginary axis grows upward.
            self.center.im - dy * step_y,
        )
    }

    /// Shift the view center by fractions of the current visible extent.
    /// `fx = 0.1` pans the view right by 10% of the horizontal extent;
    /// `fy = 0.1` pans the view up (toward larger imaginary values) by 10%
    /// of the vertical extent.
    pub fn pan(&mut self, fx: f64, fy: f64) {
        let dx = fx * 2.0 * self.scale;
        let dy = fy * 2.0 * self.scale * CELL_ASPECT;
        self.center.re += dx;
        self.center.im += dy;
    }

    /// Multiply the scale by `factor`. `< 1.0` zooms in, `> 1.0` zooms out.
    /// Scale is clamped to avoid collapsing past f64 precision.
    pub fn zoom(&mut self, factor: f64) {
        const MIN_SCALE: f64 = 1e-14;
        const MAX_SCALE: f64 = 1e3;
        self.scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_pixel_is_near_viewport_center() {
        let vp = Viewport::default_mandelbrot();
        let c = vp.pixel_to_complex(50, 15, 100, 30);
        // Cell-center offset means we land within one step of center.
        let step = 2.0 * vp.scale / 100.0;
        assert!((c.re - vp.center.re).abs() < step);
        assert!((c.im - vp.center.im).abs() < step * CELL_ASPECT);
    }

    #[test]
    fn corners_are_symmetric_around_center() {
        let vp = Viewport {
            center: Complex::new(0.0, 0.0),
            scale: 1.0,
        };
        let tl = vp.pixel_to_complex(0, 0, 100, 40);
        let br = vp.pixel_to_complex(99, 39, 100, 40);
        // Top-left should mirror bottom-right through the center.
        assert!((tl.re + br.re).abs() < 1e-9);
        assert!((tl.im + br.im).abs() < 1e-9);
        // Top-left is above and left of center.
        assert!(tl.re < 0.0 && tl.im > 0.0);
    }

    #[test]
    fn aspect_correction_keeps_cells_square_in_complex_plane() {
        let vp = Viewport {
            center: Complex::new(0.0, 0.0),
            scale: 1.0,
        };
        // Two points one column apart (horizontal step).
        let a = vp.pixel_to_complex(10, 10, 100, 40);
        let b = vp.pixel_to_complex(11, 10, 100, 40);
        let step_x = (b.re - a.re).abs();
        // Two points one row apart (vertical step).
        let c = vp.pixel_to_complex(10, 11, 100, 40);
        let step_y = (c.im - a.im).abs();
        // Vertical step should be CELL_ASPECT × horizontal step.
        assert!(((step_y / step_x) - CELL_ASPECT).abs() < 1e-9);
    }

    #[test]
    fn zoom_clamps_at_precision_limit() {
        let mut vp = Viewport::default_mandelbrot();
        for _ in 0..200 {
            vp.zoom(0.5);
        }
        assert!(vp.scale >= 1e-14);
    }
}
