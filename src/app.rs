//! Terminal lifecycle and event loop.

use std::io::{self, Write};
use std::time::Duration;

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, ClearType};

use crate::input;
use crate::render;
use crate::state::AppState;

/// RAII wrapper that enters raw mode + alternate screen on construction and
/// restores the terminal on drop. Drop runs even on panic, so the terminal
/// always returns to a usable state.
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(
            out,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(ClearType::All)
        )?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// Run the interactive event loop. Blocks until the user requests quit.
pub fn run(mut state: AppState) -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut stdout = io::stdout();

    // Sync to actual terminal size (the CLI-supplied size may be stale).
    if let Ok((cols, rows)) = terminal::size() {
        state.on_resize(cols, rows);
    }

    loop {
        if state.dirty {
            draw(&mut stdout, &state)?;
            state.dirty = false;
        }

        // Poll with a short timeout so we don't burn CPU idle-looping.
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(k) => {
                    // Ctrl+C always quits, even if the main binding is remapped.
                    if matches!(k.code, KeyCode::Char('c'))
                        && k.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    if let Some(action) = input::translate(&k) {
                        if state.apply(action) {
                            break;
                        }
                    }
                }
                Event::Resize(cols, rows) => state.on_resize(cols, rows),
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw<W: Write>(out: &mut W, state: &AppState) -> io::Result<()> {
    let (cols, rows) = state.term_size;
    if cols == 0 || rows == 0 {
        return Ok(());
    }
    // Reserve the bottom row for a status line.
    let render_rows = rows.saturating_sub(1).max(1);

    render::render(
        out,
        state.fractal.as_ref(),
        &state.viewport,
        &state.color_mode,
        state.max_iter,
        cols,
        render_rows,
    )?;

    // Status bar, positioned at the very bottom row.
    write!(
        out,
        "\x1b[{};1H\x1b[0m\x1b[2K{}",
        rows,
        truncate(&state.status_line(), cols as usize)
    )?;

    if state.show_help {
        draw_help_overlay(out)?;
    }
    out.flush()?;
    Ok(())
}

fn draw_help_overlay<W: Write>(out: &mut W) -> io::Result<()> {
    const LINES: &[&str] = &[
        " mandelbrot-cli ─ controls ",
        "                           ",
        " arrows / h j k l  pan     ",
        " Shift+arrow       fine    ",
        " + / =             zoom in ",
        " - / _             zoom out",
        " f                 fractal ",
        " [ / ]             palette ",
        " m                 mode    ",
        " g                 axis    ",
        " i / I             iter ±  ",
        " r                 reset   ",
        " ?                 close   ",
        " q / Esc           quit    ",
    ];
    // Render as a bordered box starting at (2, 2), black bg, light fg.
    for (i, line) in LINES.iter().enumerate() {
        let row = (i as u16) + 2;
        write!(
            out,
            "\x1b[{};3H\x1b[48;2;0;0;0m\x1b[38;2;230;230;230m{}\x1b[0m",
            row, line
        )?;
    }
    Ok(())
}

/// Trim a string to at most `max` visible columns. Assumes ASCII; that
/// suffices for our status content.
fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
