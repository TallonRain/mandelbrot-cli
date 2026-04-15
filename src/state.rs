//! Application state. Holds the live viewport, active fractal, color mode,
//! and related runtime flags. All mutation goes through `AppState::apply`.

use num_complex::Complex;

use crate::fractal::{Fractal, FractalKind};
use crate::input::Action;
use crate::palette::{Axis, ColorMode, Preset, Rgb};
use crate::viewport::Viewport;

/// Default accent colors used when the user toggles into position-gradient
/// mode without any prior customization.
const DEFAULT_POSITION_START: Rgb = (20, 40, 180);
const DEFAULT_POSITION_END: Rgb = (240, 120, 40);
const DEFAULT_SOLID: Rgb = (220, 220, 220);

pub struct AppState {
    pub viewport: Viewport,
    pub fractal_kind: FractalKind,
    pub fractal: Box<dyn Fractal>,
    pub julia_c: Complex<f64>,
    pub color_mode: ColorMode,
    pub max_iter: u32,
    pub term_size: (u16, u16),
    pub dirty: bool,
    pub show_help: bool,

    // Snapshot of construction values, used by Action::Reset.
    initial_viewport: Viewport,
    initial_fractal_kind: FractalKind,
    initial_color_mode: ColorMode,
    initial_max_iter: u32,
}

pub struct AppConfig {
    pub viewport: Viewport,
    pub fractal_kind: FractalKind,
    pub julia_c: Complex<f64>,
    pub color_mode: ColorMode,
    pub max_iter: u32,
    pub term_size: (u16, u16),
}

impl AppState {
    pub fn new(cfg: AppConfig) -> Self {
        let fractal = cfg.fractal_kind.build(cfg.julia_c);
        Self {
            viewport: cfg.viewport,
            fractal_kind: cfg.fractal_kind,
            fractal,
            julia_c: cfg.julia_c,
            color_mode: cfg.color_mode,
            max_iter: cfg.max_iter,
            term_size: cfg.term_size,
            dirty: true,
            show_help: false,
            initial_viewport: cfg.viewport,
            initial_fractal_kind: cfg.fractal_kind,
            initial_color_mode: cfg.color_mode,
            initial_max_iter: cfg.max_iter,
        }
    }

    /// Apply an action. Returns `true` if the event loop should quit.
    pub fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,

            Action::Pan { fx, fy } => {
                self.viewport.pan(fx, fy);
                self.dirty = true;
            }
            Action::Zoom(factor) => {
                self.viewport.zoom(factor);
                self.dirty = true;
            }
            Action::CycleFractal => {
                self.fractal_kind = self.fractal_kind.cycle_next();
                self.fractal = self.fractal_kind.build(self.julia_c);
                self.dirty = true;
            }
            Action::CyclePalette => {
                self.color_mode = match self.color_mode {
                    ColorMode::Value(p) => ColorMode::Value(p.cycle_next()),
                    _ => ColorMode::Value(Preset::Fire),
                };
                self.dirty = true;
            }
            Action::CyclePaletteBack => {
                self.color_mode = match self.color_mode {
                    ColorMode::Value(p) => ColorMode::Value(p.cycle_prev()),
                    _ => ColorMode::Value(Preset::Fire),
                };
                self.dirty = true;
            }
            Action::CycleColorMode => {
                // Preserve the "nice" value of each variant when entering it.
                self.color_mode = match self.color_mode {
                    ColorMode::Solid(_) => ColorMode::Value(Preset::Fire),
                    ColorMode::Value(_) => ColorMode::Position {
                        start: DEFAULT_POSITION_START,
                        end: DEFAULT_POSITION_END,
                        axis: Axis::Horizontal,
                    },
                    ColorMode::Position { .. } => ColorMode::Solid(DEFAULT_SOLID),
                };
                self.dirty = true;
            }
            Action::CyclePositionAxis => {
                self.color_mode = match self.color_mode {
                    ColorMode::Position { start, end, axis } => ColorMode::Position {
                        start,
                        end,
                        axis: axis.cycle_next(),
                    },
                    _ => ColorMode::Position {
                        start: DEFAULT_POSITION_START,
                        end: DEFAULT_POSITION_END,
                        axis: Axis::Horizontal,
                    },
                };
                self.dirty = true;
            }
            Action::AdjustIter(delta) => {
                let new = (self.max_iter as i32 + delta).clamp(64, 4096);
                if new as u32 != self.max_iter {
                    self.max_iter = new as u32;
                    self.dirty = true;
                }
            }
            Action::Reset => {
                self.viewport = self.initial_viewport;
                if self.fractal_kind != self.initial_fractal_kind {
                    self.fractal_kind = self.initial_fractal_kind;
                    self.fractal = self.fractal_kind.build(self.julia_c);
                }
                self.color_mode = self.initial_color_mode;
                self.max_iter = self.initial_max_iter;
                self.dirty = true;
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
                self.dirty = true;
            }
        }
        false
    }

    /// Handle a terminal resize event.
    pub fn on_resize(&mut self, cols: u16, rows: u16) {
        if (cols, rows) != self.term_size {
            self.term_size = (cols, rows);
            self.dirty = true;
        }
    }

    /// Human-readable status line for the status bar.
    pub fn status_line(&self) -> String {
        let color_desc = match self.color_mode {
            ColorMode::Solid(_) => "solid".to_string(),
            ColorMode::Value(p) => format!("value:{}", p.name()),
            ColorMode::Position { axis, .. } => format!("position:{}", axis.name()),
        };
        format!(
            "{}  scale={:.3e}  iter={}  color={}  [?] help  [q] quit",
            self.fractal.name(),
            self.viewport.scale,
            self.max_iter,
            color_desc,
        )
    }
}
