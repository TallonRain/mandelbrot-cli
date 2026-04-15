//! The Mandelbrot set: z ← z² + c, starting from z₀ = 0.

use num_complex::Complex;

use super::{Fractal, Sample, smooth_escape};

pub struct Mandelbrot;

impl Fractal for Mandelbrot {
    fn name(&self) -> &'static str {
        "Mandelbrot"
    }

    fn sample(&self, c: Complex<f64>, max_iter: u32) -> Sample {
        smooth_escape(Complex::new(0.0, 0.0), c, max_iter, |z, c| z * z + c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_inside_set() {
        let s = Mandelbrot.sample(Complex::new(0.0, 0.0), 256);
        assert_eq!(s.t, 1.0);
    }

    #[test]
    fn far_point_escapes_fast() {
        let s = Mandelbrot.sample(Complex::new(2.0, 2.0), 256);
        assert!(s.t < 0.05);
    }

    #[test]
    fn classic_inside_point_bounded() {
        // -0.75 + 0i is inside the main cardioid's period-2 bulb boundary region.
        let s = Mandelbrot.sample(Complex::new(-0.75, 0.0), 256);
        assert_eq!(s.t, 1.0);
    }
}
