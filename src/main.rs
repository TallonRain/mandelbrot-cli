mod app;
mod ascii;
mod cli;
mod fractal;
mod input;
mod palette;
mod render;
mod state;
mod viewport;

use std::io;

use anyhow::Result;
use clap::Parser;
use crossterm::terminal;

use crate::cli::Cli;
use crate::state::AppState;

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.oneshot {
        return run_oneshot(&args);
    }

    // Interactive mode: probe terminal size before entering raw mode so that
    // the initial render fills the window.
    let term_size = terminal::size().unwrap_or((100, 40));
    let config = args.to_config(term_size)?;
    let state = AppState::new(config);
    app::run(state)
}

fn run_oneshot(args: &Cli) -> Result<()> {
    let (w, h) = args.oneshot_size;
    // In oneshot mode we use the supplied grid as the full render surface
    // (no status line is reserved).
    let config = args.to_config((w, h))?;
    let fractal = config.fractal_kind.build(config.julia_c);
    let out = render::render_plain(
        fractal.as_ref(),
        &config.viewport,
        config.max_iter,
        w,
        h,
    );
    let mut stdout = io::stdout().lock();
    use std::io::Write;
    stdout.write_all(out.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}
