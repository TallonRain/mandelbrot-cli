//! Julia sets: z ← z² + c_const, with z₀ = pixel coordinate.
//!
//! Unlike the Mandelbrot set, the pixel coordinate becomes the initial z
//! value, and `c_const` is a fixed parameter that selects which Julia set
//! is being rendered. The default `c_const` is Douady's rabbit-adjacent
//! value `-0.7 + 0.27015i`.

use num_complex::Complex;

use super::{Fractal, Sample, smooth_escape};

pub struct Julia {
    pub c: Complex<f64>,
}

impl Fractal for Julia {
    fn name(&self) -> &'static str {
        "Julia"
    }

    fn sample(&self, z0: Complex<f64>, max_iter: u32) -> Sample {
        smooth_escape(z0, self.c, max_iter, |z, c| z * z + c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_point_escapes() {
        let julia = Julia {
            c: Complex::new(-0.7, 0.27015),
        };
        let s = julia.sample(Complex::new(3.0, 3.0), 256);
        assert!(s.t < 0.05);
    }

    #[test]
    fn inside_trivial_unit_disk_julia() {
        // For c = 0, the filled Julia set is the closed unit disk; any
        // starting z with |z| < 1 stays bounded under z ← z².
        let julia = Julia {
            c: Complex::new(0.0, 0.0),
        };
        let s = julia.sample(Complex::new(0.5, 0.0), 256);
        assert_eq!(s.t, 1.0);
    }
}
