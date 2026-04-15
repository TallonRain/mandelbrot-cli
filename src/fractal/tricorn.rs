//! The Tricorn (Mandelbar): z ← z̄² + c, z₀ = 0.
//!
//! Complex conjugation before squaring produces three-fold symmetry.

use num_complex::Complex;

use super::{Fractal, Sample, smooth_escape};

pub struct Tricorn;

impl Fractal for Tricorn {
    fn name(&self) -> &'static str {
        "Tricorn"
    }

    fn sample(&self, c: Complex<f64>, max_iter: u32) -> Sample {
        smooth_escape(Complex::new(0.0, 0.0), c, max_iter, |z, c| z.conj() * z.conj() + c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_inside_set() {
        let s = Tricorn.sample(Complex::new(0.0, 0.0), 256);
        assert_eq!(s.t, 1.0);
    }

    #[test]
    fn far_point_escapes() {
        let s = Tricorn.sample(Complex::new(3.0, 3.0), 256);
        assert!(s.t < 0.05);
    }
}
