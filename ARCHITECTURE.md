# ruclouds — Architecture Plan

**What it is:** A cross-platform terminal cloud renderer in Rust, in the spirit of `lavat` (metaball lava-lamp simulation) but using noise-field cloud simulation instead. Real-time, resize-adaptive, customizable via CLI flags and live keybindings.

This document is self-contained — written so that a fresh model/session with no prior context could read it and continue implementation correctly.

---

## 1. Goals & non-goals

**Goals**

- 100% cross-platform: Windows (PowerShell, pwsh, cmd.exe) and Unix (bash, Kitty, etc.) with identical behavior.
- Real-time animated clouds drifting across the terminal using noise-based simulation (not metaballs).
- Resize-adaptive: simulation is a continuous function of `(position, time)`, so resizing the terminal re-samples at new resolution without resetting or stretching existing state.
- Customizable via CLI flags at launch and live keybindings at runtime (lavat-style).
- Never leave the user's terminal in a broken state (raw mode, alt screen, cursor) on exit, panic, or Ctrl+C.

**Non-goals (v1)**

- No mouse support.
- No config file persistence (CLI flags + in-session keybindings only; can be a v2 feature).
- No metaball "puff core" hybrid layer — pure noise-field clouds for v1 (mentioned as a stretch goal only).

---

## 2. Tech stack

| Purpose | Crate | Notes |
| --- | --- | --- |
| Terminal control | `crossterm` | Raw mode, alt screen, cursor, resize events, cross-platform input polling. Do NOT use `termion` (Unix-only). |
| CLI parsing | `clap` (derive feature) | Flags + defaults |
| Noise generation | `noise` | Perlin/OpenSimplex + fBm |
| Error handling | `anyhow` | |
| RNG (seeding) | `rand` | For `--seed` / random palette / reset |

No async runtime — single-threaded fixed-timestep loop is sufficient and simpler to reason about for terminal-restore correctness.

---

## 3. File layout

```
ruclouds/
  Cargo.toml
  src/
    main.rs              - entry point: panic hook, TermGuard setup, run the App, guaranteed teardown
    app.rs                - App struct: owns Config, SimState, FrameBuffer x2 (current/previous), tick/render loop
    config.rs             - clap CLI struct, defaults, validation, RuntimeParams (the subset that's live-adjustable)
    term/
      mod.rs               - re-exports
      guard.rs             - TermGuard: enables raw mode + alt screen + hide cursor on construction, restores everything on Drop
      capability.rs        - detect truecolor / 256 / 16-color support, env var probing, --color-mode override
    sim/
      mod.rs
      noise_field.rs       - fBm sampling, domain warping, wind offset application
      wind.rs              - wind state: angle, speed, time-accumulated offset
    render/
      mod.rs
      buffer.rs             - Cell grid (RGB top/bottom per cell), resize/reallocate, diffing against previous frame
      halfblock.rs          - pack two vertical samples into one cell using ▀ + fg/bg color, emit ANSI
      palette.rs            - named palettes, hex-pair custom palettes, sky gradient, density->color mapping, ANSI downsampling (truecolor -> 256 -> 16)
    input.rs               - non-blocking crossterm event poll -> Action enum
    actions.rs              - enum of runtime actions (SpeedUp, SpeedDown, DensityUp, DensityDown, CyclePalette, ToggleWind, ToggleWarp, Reset, ToggleStorm, Quit)
  README.md
  .github/workflows/release.yml   - build matrix: windows-latest, ubuntu-latest, macos-latest
```

---

## 4. Core data structures (concrete enough to implement from)

```rust
// config.rs
struct Config {
    speed: f32,            // wind/time scroll speed, default 1.0
    density: f32,          // cloud coverage threshold, 0.0-1.0, default 0.5
    palette: PaletteSpec,  // enum: Named(String) | Custom(Color, Color)
    wind_speed: f32,
    wind_angle: f32,       // degrees
    fps: u32,              // default 30
    seed: u64,
    color_mode: ColorMode, // Auto | TrueColor | Ansi256 | Ansi16
    sky: bool,             // render sky gradient background vs transparent/terminal-bg
}

// Same fields, minus seed/color_mode/fps, are also the "live adjustable" subset
// mutated by keybindings at runtime — keep these in a separate RuntimeParams
// struct embedded in Config so input.rs can mutate it without touching launch-only fields.

// sim/noise_field.rs
struct NoiseField {
    perlin: Perlin,          // from `noise` crate, seeded
    warp_perlin: Perlin,     // second seed, for domain warping
    time: f64,               // accumulates every tick by dt * speed
    wind_offset: (f64, f64), // accumulated by wind_speed/angle every tick
}
impl NoiseField {
    // returns density in [0.0, 1.0] for a given (terminal_x, terminal_y)
    // at current sim time. This is THE function resize calls into at
    // new resolution — it must depend only on (x, y, self.time, self.wind_offset),
    // never on grid dimensions, so resampling at a new size is seamless.
    fn sample(&self, x: f64, y: f64, octaves: u32) -> f64 { ... }
}

// render/buffer.rs
struct Cell { top: Rgb, bottom: Rgb }
struct FrameBuffer {
    width: u16, height_cells: u16,  // height_cells = terminal_rows; each cell = 2 vertical samples
    cells: Vec<Cell>,
}
// Buffer is rebuilt fully every frame from NoiseField::sample, then diffed
// against the previous buffer to decide which cells need an ANSI write.
```

---

## 5. Cloud simulation algorithm (the core "what makes it look like clouds")

For each cell sub-pixel `(x, y)`:

1. **Domain warp:** sample `warp_perlin` at a slow frequency and large scale to get a small `(dx, dy)` offset. Add this to `(x, y)` before the main sample. This is what breaks up generic Perlin smoke into cloud-like wisps — do not skip this step.
2. **fBm sample:** sample `perlin` at the warped coordinates across 3-5 octaves (each octave: double frequency, halve amplitude), summed and normalized to `[0, 1]`.
3. **Wind:** before warping/sampling, offset `x` and `y` by `wind_offset`, which accumulates every tick as `wind_speed * dt` projected along `wind_angle`. This produces continuous drift.
4. **Time axis:** sample noise as 3D `(warped_x, warped_y, time)`, where `time` accumulates every tick by `dt * speed`. This is what gives smooth animation instead of popping between independent frames.
5. **Density threshold:** apply `smoothstep(density - edge_softness, density + edge_softness, noise_value)` to get cloud opacity/coverage — this is the "density" knob.
6. **Color:** blend cloud color (from palette, possibly lightened/darkened by a secondary offset-sampled "shadow" pass for puffy depth) over the sky-gradient background (vertical position based) using the opacity from step 5.

This entire pipeline is a pure function of `(x, y, time, wind_offset, config)` — critical for resize correctness (see §7).

---

## 6. Rendering pipeline (half-block technique)

- Each terminal cell = 2 vertical sub-pixel samples (top, bottom).
- Color each cell with ANSI truecolor: foreground = top color, background = bottom color, glyph = `▀` (U+2580).
- Build full `FrameBuffer` every tick from the simulation function (§5).
- Diff against previous frame's buffer; only emit cursor-move + SGR + glyph for cells that changed, to minimize I/O (important on slower Windows console paths).
- On a frame immediately following a detected resize, skip the diff and force a full repaint (old terminal content at the old size may be stale).
- Color downsampling: if `ColorMode` is not TrueColor, convert RGB to nearest 256-color or 16-color ANSI code before emitting (see `palette.rs`).

---

## 7. Resize handling (critical correctness requirement)

- Every tick: call `crossterm::terminal::size()`. If changed since last tick:
  - Reallocate `FrameBuffer` to new dimensions.
  - Do **not** touch `NoiseField::time` or `wind_offset` — they keep accumulating independent of grid size.
  - Force full repaint that frame (bypass diff).
- The simulation must never be implemented as "generate a fixed-size noise grid, then stretch/crop it on resize" — it must always be the continuous sampling function in §5 called fresh at whatever resolution the buffer currently is. This is the single most important correctness rule in this codebase.

---

## 8. Terminal lifecycle & safety

- `TermGuard` (RAII): on construction — enable raw mode, enter alternate screen, hide cursor, enable crossterm's resize-event capture. On `Drop` — reverse all of the above, unconditionally, even on panic.
- Install a panic hook in `main.rs` that runs the same teardown before re-panicking (Rust's default panic unwinding will run `Drop` impls, but a panic hook ensures teardown happens even if something later panics inside `Drop`, and lets you print a clean error after the terminal is restored rather than mid-corruption).
- Ctrl+C: handled as a normal `KeyEvent` via crossterm's input polling (no need for a separate `ctrlc` crate) — treat it as the `Quit` action so it goes through the same clean shutdown path as pressing `q`.
- Exit path (quit key, Ctrl+C, or panic) must always: disable raw mode → leave alt screen → show cursor → flush stdout, in that order, before the process exits.

---

## 9. Input / live keybindings

Non-blocking poll each tick (`crossterm::event::poll` with ~0 timeout, so it never blocks the render loop). Map to `Action` enum, mutate `Config.runtime` fields directly:

| Key | Action |
|---|---|
| `+` / `-` | Speed up / down |
| `[` / `]` | Density down / up |
| `c` | Cycle palette |
| `w` | Toggle/cycle wind direction |
| `g` | Toggle domain-warp intensity (subtle vs strong) |
| `r` | Reseed/reset (new seed, time resets to 0) |
| `p` | Storm mode toggle (temporarily boosts speed + density) |
| `q` / `Esc` | Quit |

---

## 10. CLI flags (v1 set)

```
--speed <f32>            default 1.0
--density <f32 0.0-1.0>  default 0.5
--palette <name|hex,hex> default "white-grey"
--wind-speed <f32>       default 0.3
--wind-angle <f32 deg>   default 0.0  (0 = left-to-right)
--fps <u32>              default 30
--seed <u64>             default: random
--color-mode <auto|truecolor|256|ansi16>  default auto
--no-sky                 flag, renders against terminal's existing background instead of a sky gradient
```

---

## 11. Performance notes

- Noise sampling is the hot path. Precompute octave frequency/amplitude tables once at startup, not per-sample.
- Fixed timestep loop: compute frame budget from `fps`, sleep remainder, skip sleep (don't accumulate debt) if behind.
- If profiling shows noise sampling too slow at high terminal resolution + many octaves, first lever to pull is reducing octave count before reducing resolution.

---

## 12. Cross-platform test matrix

- Windows: PowerShell, pwsh (PowerShell 7), cmd.exe — verify truecolor on PowerShell/pwsh, verify graceful 256/16-color fallback detection on cmd.exe if it misreports capability.
- Unix: bash + Kitty (Linux/Debian) — verify truecolor, verify resize-in-place works without flicker/garbage.
- All platforms: verify Ctrl+C and panic (e.g. temporarily force a `panic!()` mid-run) both leave the terminal in a clean, usable state afterward.
- All platforms: resize the terminal window mid-run repeatedly, both growing and shrinking, confirm no stretching/cropping artifacts and no crash.

---

## 13. Build / distribution

- `cargo install ruclouds` via crates.io (optional, once stable).
- GitHub Actions matrix (`windows-latest`, `ubuntu-latest`, `macos-latest`) building release binaries per platform on tag push.

---

## 14. Suggested implementation order (milestones)

1. `TermGuard` + panic hook + minimal event loop that just clears the screen and quits on `q`. Verify terminal restore works correctly on all target shells before writing any rendering code.
2. `FrameBuffer` + half-block renderer rendering a static test pattern (e.g. solid color gradient) to validate the ANSI emission and diffing logic.
3. `NoiseField` sampling wired into the buffer, no wind/warp yet — confirm clouds animate via the time axis.
4. Add domain warping, wind drift, palette/sky gradient blending.
5. Wire up CLI flags (`config.rs` + `clap`).
6. Wire up live keybindings (`input.rs` + `actions.rs`).
7. Resize handling + forced full-repaint-on-resize.
8. Color-mode fallback (256/16-color downsampling) + capability detection.
9. Cross-platform test pass per §12.
10. README + GitHub Actions release workflow.