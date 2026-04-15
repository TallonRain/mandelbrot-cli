# mandelbrot-cli

An interactive ASCII fractal viewer for the terminal, written in Rust. Renders
the Mandelbrot set (and four other fractals) as colored character art, with
live keyboard-driven zoom, pan, and color-mode switching. Pixel computation
runs in parallel via [rayon](https://crates.io/crates/rayon).

## Features

- Five fractals: **Mandelbrot** (default), **Julia**, **Burning Ship**,
  **Tricorn**, and **Newton** (z³ − 1).
- Three color modes, switchable at runtime:
  - **Solid** — a single 24-bit color with density carried by the ASCII ramp.
  - **Value** — iteration count mapped to a palette (`fire`, `ocean`,
    `grayscale`, `rainbow`, `electric`). Classic fractal-coloring look.
  - **Position** — two endpoint colors blended across the terminal window
    along a horizontal, vertical, or diagonal axis.
- Interactive keyboard controls: pan, zoom, cycle fractals/palettes/modes,
  adjust max iterations, reset, help overlay, quit.
- Multithreaded per-row computation with rayon.
- Aspect-corrected rendering (terminal glyphs are ~2:1 tall:wide).
- Smooth escape-time coloring (no visible banding).
- RAII terminal cleanup — the terminal always returns to a usable state,
  even on panic or `Ctrl+C`.
- Hidden `--oneshot` mode for CI and scripting.

## Requirements

- Rust (edition 2024, so a recent stable toolchain — 1.85 or newer).
- A terminal with 24-bit true color and ANSI escape support. On Windows the
  built-in Windows Terminal, VS Code's integrated terminal, and modern
  ConPTY-backed shells all work. Legacy `cmd.exe` will render but with
  limited color fidelity.

## Build and run

```
cargo build --release
cargo run --release
```

Launch with defaults (Mandelbrot, value-gradient with the `fire` palette):

```
cargo run --release
```

Launch with a specific configuration:

```
cargo run --release -- --fractal julia --palette ocean --max-iter 512
```

## Controls

Press `?` at any time to toggle an in-app help overlay.

| Keys                 | Action                                   |
| -------------------- | ---------------------------------------- |
| Arrow keys / `h j k l` | Pan by 10% of the visible extent       |
| Shift + arrow key    | Fine pan (1%)                            |
| `+` / `=`            | Zoom in (×0.8)                           |
| `-` / `_`            | Zoom out (×1.25)                         |
| `f`                  | Cycle through fractals                   |
| `[` / `]`            | Previous / next palette (value mode)     |
| `m`                  | Cycle color mode (solid → value → position) |
| `g`                  | Cycle position-gradient axis (H / V / diag) |
| `i` / `I`            | Decrease / increase `max_iter` by 64     |
| `r`                  | Reset to the initial view                |
| `?`                  | Toggle help overlay                      |
| `q` / `Esc`          | Quit                                     |
| `Ctrl + C`           | Quit (always, even if remapped)          |

Pressing `[`, `]`, or `g` while in a color mode that does not use that
concept (for example, `[` while in solid mode) switches into the appropriate
mode with a default value.

## CLI options

```
mandelbrot-cli [OPTIONS]
```

| Flag              | Type                       | Default        | Description                                            |
| ----------------- | -------------------------- | -------------- | ------------------------------------------------------ |
| `--fractal`       | `mandelbrot` \| `julia` \| `burning-ship` \| `tricorn` \| `newton` | `mandelbrot` | Which fractal to render on startup.                    |
| `--palette`       | `fire` \| `ocean` \| `grayscale` \| `rainbow` \| `electric` | `fire`         | Palette used when `--color-mode=value`.                |
| `--color`         | `#RRGGBB`                  | `#DCDCDC`      | Solid color used when `--color-mode=solid`.            |
| `--color-mode`    | `solid` \| `value` \| `position` | `value`     | Initial color mode.                                    |
| `--max-iter`      | integer (64..=4096)        | `256`          | Maximum iterations per pixel.                          |
| `--center`        | `re,im`                    | fractal-specific | Initial view center on the complex plane.              |
| `--zoom`          | floating-point             | fractal-specific | Initial horizontal half-width of the visible region. Smaller = more zoomed in. |
| `--julia-c`       | `re,im`                    | `-0.7,0.27015` | The constant `c` for the Julia set.                    |
| `-h` / `--help`   | —                          | —              | Print CLI help.                                        |
| `-V` / `--version`| —                          | —              | Print version.                                         |

### Passing negative numbers

`clap` interprets leading `-` as the start of a flag. To pass a negative
value, use the `=` form:

```
mandelbrot-cli --center=-0.743,0.131 --zoom 0.005
```

### A few interesting starting points

Deep zoom into the seahorse valley:

```
mandelbrot-cli --center=-0.743643887037151,0.131825904205330 --zoom 0.0005 --max-iter 1024
```

Douady's rabbit Julia set:

```
mandelbrot-cli --fractal julia --julia-c=-0.123,0.745 --palette electric
```

Burning Ship's main hull:

```
mandelbrot-cli --fractal burning-ship --center=-1.755,-0.03 --zoom 0.1
```

Newton basins:

```
mandelbrot-cli --fractal newton --zoom 2.0 --max-iter 64
```

## How it works

The pipeline for a single frame:

1. **Compute.** For each terminal cell (column, row), the viewport transforms
   the pixel coordinate into a point on the complex plane (applying a 2×
   vertical correction so circles stay round despite non-square cells).
   That point is handed to the active fractal, which iterates until the
   sequence escapes a bailout radius or a root is reached. The result is a
   `Sample { t, class }`: `t ∈ [0, 1]` for density, `class > 0` only for
   Newton basins.
2. **Color and shape.** `t` picks a glyph from the density ramp
   `" .:-=+*#%@"` (smooth escape-time formula avoids banding). The active
   color mode picks a 24-bit RGB color:
   - Solid mode: a fixed color scaled by `t`.
   - Value mode: palette interpolated in linear-light sRGB by `t`.
   - Position mode: two endpoint colors interpolated by the cell's position
     in the window, independent of `t`.
   Newton's basin index overrides the color mode so the three roots show as
   red, green, and blue basins, modulated by convergence speed.
3. **Write.** Rows are computed in parallel with
   `rayon::iter::IntoParallelIterator`, then serialized to a single buffer
   of ANSI escapes and written to `stdout` in one call.

Terminal state is guarded by a `TerminalGuard` that enters raw mode and the
alternate screen on construction and restores them on drop, so panics and
unexpected exits leave the terminal usable.

## Module layout

```
src/
├── main.rs            Entry point; dispatches interactive vs --oneshot
├── cli.rs             clap parser and AppConfig construction
├── app.rs             Event loop and TerminalGuard
├── state.rs           AppState and apply(Action) mutation
├── viewport.rs        Complex-plane coordinate transform
├── render.rs          Parallel compute and ANSI output
├── palette.rs         Color modes and palette presets
├── ascii.rs           Density ramp → glyph
├── input.rs           KeyEvent → Action translator
└── fractal/
    ├── mod.rs         Fractal trait + smooth_escape helper
    ├── mandelbrot.rs
    ├── julia.rs
    ├── burning_ship.rs
    ├── tricorn.rs
    └── newton.rs
```

## Development

Run the test suite:

```
cargo test
```

Tests cover:
- ASCII ramp boundary behavior.
- Viewport coordinate round-trips and aspect correction.
- Each fractal's inside-set and outside-set sample behavior.
- Palette endpoints, monotonic grayscale luminance, Newton basin tinting.

### Oneshot mode (CI-friendly)

`--oneshot` renders a single plain-ASCII frame (no color escapes) to stdout
and exits. Useful for smoke tests and snapshot comparisons:

```
cargo run --release -- --oneshot --oneshot-size 80x24 --fractal mandelbrot
```

Both `--oneshot` and `--oneshot-size` are hidden from `--help` so they don't
clutter the normal usage output.

## Limitations

- **Precision.** All math is `f64`. Zoom is clamped at `1e-14`, past which
  the Mandelbrot's detail dissolves into numerical noise. Arbitrary-precision
  arithmetic is not implemented.
- **No mouse support.** Pan and zoom are keyboard-only in this version.
- **No image export.** The renderer targets `stdout` for an interactive
  terminal; there is no option to save a frame as an image file.
- **No config file.** Startup is configured entirely via CLI flags; runtime
  tweaks are via keybindings and are not persisted between sessions.
