//! Translates crossterm key events into application `Action`s.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// Pan the viewport; `fx` / `fy` are fractions of current visible extent.
    Pan { fx: f64, fy: f64 },
    /// Multiplicative zoom factor; `< 1.0` zooms in, `> 1.0` zooms out.
    Zoom(f64),
    CycleFractal,
    CyclePalette,
    CyclePaletteBack,
    CycleColorMode,
    CyclePositionAxis,
    /// Adjust `max_iter` by this many steps (negative = decrease).
    AdjustIter(i32),
    Reset,
    ToggleHelp,
    Quit,
}

/// Convert a key event into an optional `Action`. Unrecognized keys return
/// `None` so the event loop can ignore them.
pub fn translate(ev: &KeyEvent) -> Option<Action> {
    use KeyCode::*;

    let shift = ev.modifiers.contains(KeyModifiers::SHIFT);
    // Shift = fine pan (1%); unmodified = normal pan (10%).
    let step = if shift { 0.01 } else { 0.1 };

    Some(match ev.code {
        Char('q') | Esc => Action::Quit,

        Left | Char('h') => Action::Pan { fx: -step, fy: 0.0 },
        Right | Char('l') => Action::Pan { fx: step, fy: 0.0 },
        Up | Char('k') => Action::Pan { fx: 0.0, fy: step },
        Down | Char('j') => Action::Pan { fx: 0.0, fy: -step },

        Char('+') | Char('=') => Action::Zoom(0.8),
        Char('-') | Char('_') => Action::Zoom(1.25),

        Char('f') => Action::CycleFractal,
        Char(']') => Action::CyclePalette,
        Char('[') => Action::CyclePaletteBack,
        Char('m') => Action::CycleColorMode,
        Char('g') => Action::CyclePositionAxis,

        Char('i') => Action::AdjustIter(-64),
        Char('I') => Action::AdjustIter(64),

        Char('r') => Action::Reset,
        Char('?') => Action::ToggleHelp,

        _ => return None,
    })
}
