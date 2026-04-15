//! Command-line argument parsing. `Cli::to_config` converts the parsed
//! arguments into an `AppConfig` consumed by `AppState::new`.

use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum};
use num_complex::Complex;

use crate::fractal::FractalKind;
use crate::palette::{Axis, ColorMode, Preset, Rgb};
use crate::state::AppConfig;
use crate::viewport::Viewport;

#[derive(Parser, Debug)]
#[command(
    name = "mandelbrot-cli",
    about = "Interactive ASCII fractal viewer (Mandelbrot, Julia, Burning Ship, Tricorn, Newton)",
    version,
)]
pub struct Cli {
    /// Which fractal to render.
    #[arg(long, value_enum, default_value_t = FractalArg::Mandelbrot)]
    pub fractal: FractalArg,

    /// Preset palette for value-gradient mode.
    #[arg(long, value_enum)]
    pub palette: Option<PaletteArg>,

    /// Solid color used when --color-mode=solid (format: #RRGGBB).
    #[arg(long)]
    pub color: Option<String>,

    /// Initial color mode.
    #[arg(long, value_enum, default_value_t = ColorModeArg::Value)]
    pub color_mode: ColorModeArg,

    /// Maximum iterations per pixel (64..=4096).
    #[arg(long, default_value_t = 256)]
    pub max_iter: u32,

    /// Initial view center as "re,im".
    #[arg(long, value_parser = parse_complex)]
    pub center: Option<Complex<f64>>,

    /// Initial horizontal half-width of the complex plane (smaller = more zoomed in).
    #[arg(long)]
    pub zoom: Option<f64>,

    /// Julia-set parameter "c" as "re,im".
    #[arg(long, value_parser = parse_complex)]
    pub julia_c: Option<Complex<f64>>,

    /// Render a single frame to stdout and exit. Used for CI/smoke tests.
    #[arg(long, hide = true)]
    pub oneshot: bool,

    /// Grid size for --oneshot mode (format: WIDTHxHEIGHT).
    #[arg(long, value_parser = parse_size, default_value = "80x24", hide = true)]
    pub oneshot_size: (u16, u16),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FractalArg {
    Mandelbrot,
    Julia,
    #[value(name = "burning-ship")]
    BurningShip,
    Tricorn,
    Newton,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PaletteArg {
    Fire,
    Ocean,
    Grayscale,
    Rainbow,
    Electric,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorModeArg {
    Solid,
    Value,
    Position,
}

impl Cli {
    /// Convert parsed CLI into an `AppConfig`. `term_size` is the current
    /// terminal size (columns × rows).
    pub fn to_config(&self, term_size: (u16, u16)) -> Result<AppConfig> {
        let fractal_kind = self.fractal.kind();
        let julia_c = self
            .julia_c
            .unwrap_or_else(|| Complex::new(-0.7, 0.27015));

        let preset = self.palette.map(PaletteArg::preset).unwrap_or(Preset::Fire);

        let solid = match &self.color {
            Some(s) => parse_hex_color(s)?,
            None => (220, 220, 220),
        };

        let color_mode = match self.color_mode {
            ColorModeArg::Solid => ColorMode::Solid(solid),
            ColorModeArg::Value => ColorMode::Value(preset),
            ColorModeArg::Position => ColorMode::Position {
                start: (20, 40, 180),
                end: (240, 120, 40),
                axis: Axis::Horizontal,
            },
        };

        let mut viewport = default_viewport_for(fractal_kind);
        if let Some(c) = self.center {
            viewport.center = c;
        }
        if let Some(z) = self.zoom {
            viewport.scale = z;
        }

        let max_iter = self.max_iter.clamp(64, 4096);

        Ok(AppConfig {
            viewport,
            fractal_kind,
            julia_c,
            color_mode,
            max_iter,
            term_size,
        })
    }
}

impl FractalArg {
    fn kind(self) -> FractalKind {
        match self {
            FractalArg::Mandelbrot => FractalKind::Mandelbrot,
            FractalArg::Julia => FractalKind::Julia,
            FractalArg::BurningShip => FractalKind::BurningShip,
            FractalArg::Tricorn => FractalKind::Tricorn,
            FractalArg::Newton => FractalKind::Newton,
        }
    }
}

impl PaletteArg {
    fn preset(self) -> Preset {
        match self {
            PaletteArg::Fire => Preset::Fire,
            PaletteArg::Ocean => Preset::Ocean,
            PaletteArg::Grayscale => Preset::Grayscale,
            PaletteArg::Rainbow => Preset::Rainbow,
            PaletteArg::Electric => Preset::Electric,
        }
    }
}

fn default_viewport_for(kind: FractalKind) -> Viewport {
    match kind {
        FractalKind::Mandelbrot => Viewport::default_mandelbrot(),
        FractalKind::Julia | FractalKind::Newton => Viewport {
            center: Complex::new(0.0, 0.0),
            scale: 1.5,
        },
        FractalKind::BurningShip => Viewport {
            center: Complex::new(-0.5, -0.5),
            scale: 1.5,
        },
        FractalKind::Tricorn => Viewport {
            center: Complex::new(0.0, 0.0),
            scale: 2.0,
        },
    }
}

fn parse_complex(s: &str) -> std::result::Result<Complex<f64>, String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("expected 're,im', got '{s}'"));
    }
    let re: f64 = parts[0]
        .trim()
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())?;
    let im: f64 = parts[1]
        .trim()
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())?;
    Ok(Complex::new(re, im))
}

fn parse_size(s: &str) -> std::result::Result<(u16, u16), String> {
    let parts: Vec<&str> = s.split(['x', 'X']).collect();
    if parts.len() != 2 {
        return Err(format!("expected WIDTHxHEIGHT, got '{s}'"));
    }
    let w: u16 = parts[0]
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;
    let h: u16 = parts[1]
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;
    if w == 0 || h == 0 {
        return Err("width and height must be > 0".into());
    }
    Ok((w, h))
}

fn parse_hex_color(s: &str) -> Result<Rgb> {
    let t = s.trim_start_matches('#');
    if t.len() != 6 {
        return Err(anyhow!("expected #RRGGBB, got '{s}'"));
    }
    let r = u8::from_str_radix(&t[0..2], 16)?;
    let g = u8::from_str_radix(&t[2..4], 16)?;
    let b = u8::from_str_radix(&t[4..6], 16)?;
    Ok((r, g, b))
}
