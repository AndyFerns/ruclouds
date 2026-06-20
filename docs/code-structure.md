# Code Structure

This section explains the architecture and organization of the ruclouds codebase.

## Project Layout

```
ruclouds/
├── Cargo.toml              # Project metadata and dependencies
├── Cargo.lock              # Dependency lock file
├── README.md              # User-facing documentation
├── ARCHITECTURE.md        # Detailed architecture documentation
├── mkdocs.yml             # MkDocs configuration
└── src/
    ├── main.rs            # Entry point with panic hook and TermGuard
    ├── app.rs             # Main application loop and orchestration
    ├── config.rs          # CLI parsing and runtime configuration
    ├── actions.rs         # Action enumeration for input handling
    ├── input.rs           # Non-blocking input polling
    ├── sim/               # Cloud simulation module
    │   ├── mod.rs         # Simulation orchestration
    │   ├── noise_field.rs # Perlin noise with fBm and domain warp
    │   └── wind.rs        # Wind offset management
    ├── render/            # Rendering module
    │   ├── mod.rs         # Renderer orchestration
    │   ├── buffer.rs      # Frame buffer management
    │   ├── halfblock.rs   # Cell and half-block rendering
    │   └── palette.rs     # Color palettes and blending
    └── term/              # Terminal module
        ├── mod.rs         # Module re-exports
        ├── guard.rs       # Terminal state RAII guard
        └── capability.rs  # Color capability detection
```

## Module Responsibilities

### main.rs

Entry point that sets up the panic hook and creates the TermGuard before running the application. This ensures terminal cleanup happens even on panic.

Key responsibilities:
- Install panic hook for terminal cleanup
- Parse CLI arguments
- Create TermGuard (RAII)
- Run the main application loop

### app.rs

Main application loop that orchestrates the simulation, rendering, and input handling.

Key responsibilities:
- Resolve CLI arguments (seed, palette, color mode)
- Initialize Simulation and Renderer
- Main render loop with fixed timestep
- Handle terminal resize events
- Process input actions
- Apply runtime parameter changes

### config.rs

Handles CLI argument parsing and runtime configuration.

Key responsibilities:
- Parse CLI flags using clap
- Define ColorMode enum
- Provide Config struct with runtime-adjustable parameters
- Compute effective values (storm mode boosts, warp intensity)

### actions.rs

Simple enumeration of all possible runtime actions triggered by keyboard input.

Key responsibilities:
- Define Action enum variants
- No logic, just data structure

### input.rs

Non-blocking input polling that maps keyboard events to actions.

Key responsibilities:
- Poll for input without blocking
- Map key events to Action enum
- Handle platform-specific key event differences (Windows Press/Repeat/Release)

### sim/mod.rs

Simulation orchestration that combines noise field and wind into a coherent cloud simulation.

Key responsibilities:
- Define Simulation struct
- Implement update loop
- Provide sample_pixel function (pure function of position and time)
- Handle reseeding

Key invariant: sample_pixel depends only on (px, py, time, wind_offset, config) - never on grid dimensions. This is critical for resize correctness.

### sim/noise_field.rs

Perlin noise with fractal Brownian motion (fBm) and domain warping.

Key responsibilities:
- Manage primary and warp Perlin noise generators
- Precompute octave frequency/amplitude tables
- Sample fBm at 3D coordinates
- Compute domain-warp displacement

The domain warp is what creates wispy cloud shapes instead of generic Perlin smoke.

### sim/wind.rs

Manages wind offset that drifts the noise-field coordinates.

Key responsibilities:
- Track accumulated wind offset
- Update offset based on speed and angle each timestep

Note: Wind offset is NOT reset when reseeding - this preserves spatial continuity.

### render/mod.rs

Renderer orchestration that computes frames, diffs against previous frame, and emits ANSI commands.

Key responsibilities:
- Manage double-buffered FrameBuffers
- Compute current frame from simulation
- Diff against previous frame
- Emit only changed cells as ANSI commands
- Handle force-full repaint on resize

The diffing minimizes terminal I/O, which is especially important on Windows.

### render/buffer.rs

Frame buffer that stores a grid of cells representing one complete frame.

Key responsibilities:
- Allocate and resize cell grid
- Provide get/set accessors
- Clear buffer on resize

### render/halfblock.rs

Defines the Cell struct and half-block character constant.

Key responsibilities:
- Define Cell with top/bottom RGB colors
- Provide same_as method for diffing
- Define UPPER_HALF_BLOCK constant

### render/palette.rs

Color management including palettes, blending, and color mode conversion.

Key responsibilities:
- Define built-in palettes
- Parse custom hex palettes
- Implement color blending (linear interpolation)
- Convert RGB to 256-color and ANSI-16
- Map RGB to crossterm Color based on mode

### term/mod.rs

Module re-exports for the terminal module.

### term/guard.rs

RAII guard that sets up and tears down terminal state.

Key responsibilities:
- Enable raw mode on construction
- Enter alternate screen on construction
- Hide cursor on construction
- Reverse all of above on Drop
- Provide teardown function for panic hook

This ensures terminal state is always restored, even on panic or Ctrl+C.

### term/capability.rs

Detects terminal color capability from environment variables.

Key responsibilities:
- Check COLORTERM for truecolor support
- Check WT_SESSION for Windows Terminal
- Check TERM_PROGRAM for known terminals
- Check TERM for kitty/alacritty/256color
- Provide safe fallback (256-color)

## Data Flow

### Initialization Flow

1. main.rs installs panic hook
2. main.rs parses CLI args
3. main.rs creates TermGuard (enables raw mode, alt screen, hides cursor)
4. app.rs resolves seed, palette, color mode
5. app.rs creates Simulation and Renderer
6. app.rs enters main loop

### Per-Frame Flow

1. Check terminal size, resize if changed
2. Poll input, drain all pending actions
3. Apply actions to Config
4. Update Simulation (advance time and wind)
5. Renderer computes frame from Simulation
6. Renderer diffs against previous frame
7. Renderer emits ANSI commands for changed cells
8. Swap buffers (current becomes previous)
9. Sleep remaining frame budget

### Resize Flow

1. Detect size change
2. Resize both FrameBuffers
3. Force full repaint (skip diff)
4. Continue normal flow

Simulation time and wind offset are NOT reset on resize - this preserves continuity.

## Key Design Decisions

### Pure Function Sampling

The simulation sample_pixel function is a pure function of (px, py, time, wind_offset, config). It never depends on grid dimensions. This is what makes resize seamless - the simulation is always re-sampled fresh at whatever resolution the buffer currently is.

### Double-Buffer Diffing

The renderer maintains two frame buffers (current and previous). Only cells that changed emit ANSI writes. This minimizes terminal I/O, which is critical for performance on Windows.

### RAII Terminal Guard

Terminal state is managed by a RAII guard that restores everything on Drop. Combined with a panic hook, this guarantees the terminal is never left in a broken state.

### Fixed Timestep Loop

The render loop uses fixed timestep (dt = 1/fps) with frame budgeting. If behind, it skips sleep (no debt accumulation). This prevents spiral of death.

### Domain Warping

Domain warping bends the coordinate space before noise sampling, creating wispy cloud shapes instead of generic Perlin smoke. This is a critical visual feature.

### Half-Block Technique

Each terminal cell represents two vertical sub-pixels using the ▀ character. Foreground is top, background is bottom. This doubles effective vertical resolution.

## Performance Considerations

### Noise Sampling

Noise sampling is the hot path. Octave frequency/amplitude tables are precomputed at startup, not per-sample.

### Frame Diffing

Only changed cells emit ANSI writes. This is especially important on Windows where console I/O is slower.

### Color Mode Fallback

Color conversion (truecolor → 256 → 16) happens once per cell per frame, not per-sample.

### Memory Allocation

Frame buffers are pre-allocated and reused. Resize reallocates but doesn't allocate per-frame.

## Cross-Platform Considerations

### Windows

- crossterm handles Windows console API
- Windows Terminal supports truecolor
- Older cmd.exe may need 256 or 16-color fallback
- Input handling filters Press/Repeat/Release events

### Unix

- Most modern terminals support truecolor
- TERM and COLORTERM environment variables indicate capability
- Resize events work reliably

### Common

- Same code path for both platforms
- crossterm abstracts platform differences
- Panic hook and RAII guard work identically

## Testing Strategy

### Manual Testing

- Terminal restore on quit, Ctrl+C, and panic
- Resize handling (grow and shrink)
- Color mode detection and fallback
- All keybindings
- All CLI flags

### Platform Testing

- Windows: PowerShell, pwsh, cmd.exe
- Unix: bash with Kitty, Alacritty, iTerm2
- macOS: Terminal.app, iTerm2

## Future Extensions

Potential areas for expansion (not implemented in v1):

- Config file persistence
- Mouse support
- Additional noise algorithms
- More built-in palettes
- Cloud layering (multiple noise fields)
- Performance profiling and optimization
