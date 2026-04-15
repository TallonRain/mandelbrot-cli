//! Newton fractal for the polynomial z³ − 1. Each pixel is iterated under
//! Newton's method until it converges to one of the three cube roots of
//! unity; the basin is returned as `class`, and `t` encodes convergence
//! speed (fast convergence → high `t`, so roots render as the densest char).

use num_complex::Complex;

use super::{Fractal, Sample};

/// √3 / 2 — the imaginary component of the non-real cube roots of unity.
const SQRT3_OVER_2: f64 = 0.866_025_403_784_438_6;

/// Squared convergence tolerance. Once `|z - root|² < EPS_SQ`, we consider
/// the point to have landed in that root's basin.
const EPS_SQ: f64 = 1e-12;

pub struct Newton;

impl Fractal for Newton {
    fn name(&self) -> &'static str {
        "Newton (z³−1)"
    }

    fn sample(&self, z0: Complex<f64>, max_iter: u32) -> Sample {
        let roots = [
            Complex::new(1.0, 0.0),
            Complex::new(-0.5, SQRT3_OVER_2),
            Complex::new(-0.5, -SQRT3_OVER_2),
        ];
        let mut z = z0;
        for n in 0..max_iter {
            for (i, root) in roots.iter().enumerate() {
                if (z - *root).norm_sqr() < EPS_SQ {
                    // Fast convergence → high t → dense glyph + bright color.
                    let t = 1.0 - (n as f64 / max_iter as f64);
                    return Sample {
                        t,
                        class: (i + 1) as u8,
                    };
                }
            }
            let z2 = z * z;
            if z2.norm_sqr() < 1e-30 {
                // Derivative vanishes — bail.
                break;
            }
            // z ← z − (z³ − 1) / (3 z²)  =  (2z³ + 1) / (3 z²)
            let z3 = z2 * z;
            z = (Complex::new(2.0, 0.0) * z3 + Complex::new(1.0, 0.0))
                / (Complex::new(3.0, 0.0) * z2);
        }
        // Did not converge: treat as boundary / no-basin.
        Sample { t: 0.0, class: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_one_is_its_own_basin() {
        let s = Newton.sample(Complex::new(1.0, 0.0), 64);
        assert_eq!(s.class, 1);
        assert!(s.t > 0.9);
    }

    #[test]
    fn near_second_root_converges_to_basin_two() {
        let near = Complex::new(-0.5 + 0.01, SQRT3_OVER_2 + 0.01);
        let s = Newton.sample(near, 256);
        assert_eq!(s.class, 2);
    }

    #[test]
    fn near_third_root_converges_to_basin_three() {
        let near = Complex::new(-0.5 + 0.01, -SQRT3_OVER_2 - 0.01);
        let s = Newton.sample(near, 256);
        assert_eq!(s.class, 3);
    }
}
