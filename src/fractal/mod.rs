//! Fractal trait and registry. Each fractal maps a complex-plane point to a
//! normalized `Sample` used by rendering for both density and color.

use num_complex::Complex;

pub mod burning_ship;
pub mod julia;
pub mod mandelbrot;
pub mod newton;
pub mod tricorn;

pub use burning_ship::BurningShip;
pub use julia::Julia;
pub use mandelbrot::Mandelbrot;
pub use newton::Newton;
pub use tricorn::Tricorn;

/// Result of iterating a fractal at a single point.
///
/// `t` is a normalized value in `[0.0, 1.0]` used for ASCII density. For
/// escape-time fractals this is the smoothed iteration count / max_iter,
/// saturated at 1.0 inside the set. For Newton it reflects convergence speed.
///
/// `class` is an optional discrete tag for fractals with multiple basins of
/// attraction (Newton). Escape-time fractals always return `0`.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub t: f64,
    pub class: u8,
}

/// Trait implemented by all renderable fractals. Implementors must be
/// `Send + Sync` because rendering parallelizes across pixels with rayon.
pub trait Fractal: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample(&self, c: Complex<f64>, max_iter: u32) -> Sample;
}

/// Identifier for the built-in fractals. Used by CLI parsing and runtime
/// cycling. Keep the order here stable — `cycle_next` relies on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalKind {
    Mandelbrot,
    Julia,
    BurningShip,
    Tricorn,
    Newton,
}

impl FractalKind {
    pub const ALL: &'static [FractalKind] = &[
        FractalKind::Mandelbrot,
        FractalKind::Julia,
        FractalKind::BurningShip,
        FractalKind::Tricorn,
        FractalKind::Newton,
    ];

    pub fn cycle_next(self) -> FractalKind {
        let all = Self::ALL;
        let idx = all.iter().position(|&k| k == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    /// Build a boxed fractal for this kind. `julia_c` is consumed only by Julia.
    pub fn build(self, julia_c: Complex<f64>) -> Box<dyn Fractal> {
        match self {
            FractalKind::Mandelbrot => Box::new(Mandelbrot),
            FractalKind::Julia => Box::new(Julia { c: julia_c }),
            FractalKind::BurningShip => Box::new(BurningShip),
            FractalKind::Tricorn => Box::new(Tricorn),
            FractalKind::Newton => Box::new(Newton),
        }
    }
}

/// Shared escape-time helper: returns the smoothed iteration count for a
/// quadratic-family fractal. Caller supplies the iteration function.
///
/// Smoothing formula: `mu = n + 1 - log2(log(|z|))`. This eliminates banding
/// artifacts from discrete iteration counts.
pub(crate) fn smooth_escape<F>(
    mut z: Complex<f64>,
    c: Complex<f64>,
    max_iter: u32,
    mut step: F,
) -> Sample
where
    F: FnMut(Complex<f64>, Complex<f64>) -> Complex<f64>,
{
    const BAILOUT: f64 = 4.0;
    for n in 0..max_iter {
        if z.norm_sqr() > BAILOUT {
            let log_zn = z.norm_sqr().ln() * 0.5;
            let nu = (log_zn / 2f64.ln()).ln() / 2f64.ln();
            let mu = (n as f64 + 1.0 - nu).max(0.0);
            let t = (mu / max_iter as f64).clamp(0.0, 1.0);
            return Sample { t, class: 0 };
        }
        z = step(z, c);
    }
    // Inside the set.
    Sample { t: 1.0, class: 0 }
}
