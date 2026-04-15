//! The Burning Ship: z ← (|Re(z)| + i·|Im(z)|)² + c, z₀ = 0.
//!
//! Absolute values applied before squaring produce the set's characteristic
//! ship-like silhouette.

use num_complex::Complex;

use super::{Fractal, Sample, smooth_escape};

pub struct BurningShip;

impl Fractal for BurningShip {
    fn name(&self) -> &'static str {
        "Burning Ship"
    }

    fn sample(&self, c: Complex<f64>, max_iter: u32) -> Sample {
        smooth_escape(Complex::new(0.0, 0.0), c, max_iter, |z, c| {
            let z = Complex::new(z.re.abs(), z.im.abs());
            z * z + c
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_point_escapes() {
        let s = BurningShip.sample(Complex::new(5.0, 5.0), 256);
        assert!(s.t < 0.05);
    }

    #[test]
    fn origin_is_inside_set() {
        // With c = 0 + 0i, z starts at 0 and the iteration is stationary.
        let s = BurningShip.sample(Complex::new(0.0, 0.0), 256);
        assert_eq!(s.t, 1.0);
    }
}
