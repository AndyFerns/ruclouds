# API Reference

This section covers the main API components and their usage. ruclouds is primarily designed as a CLI application, but the internal modules are structured to be modular and potentially reusable.

## Core Modules

### Config Module (`src/config.rs`)

The config module handles CLI argument parsing and runtime configuration.

#### `CliArgs`

CLI arguments parsed by clap:

```rust
pub struct CliArgs {
    pub speed: f32,           // Animation speed multiplier
    pub density: f32,         // Cloud density threshold
    pub palette: String,      // Palette name or custom hex pair
    pub wind_speed: f32,      // Wind speed
    pub wind_angle: f32,      // Wind angle in degrees
    pub fps: u32,             // Target frames per second
    pub seed: Option<u64>,    // Random seed
    pub color_mode: ColorMode, // Color mode
    pub no_sky: bool,         // Use terminal background
}
```

#### `Config`

Runtime configuration that includes both CLI args and runtime-adjustable parameters:

```rust
pub struct Config {
    pub speed: f32,
    pub density: f32,
    pub edge_softness: f32,
    pub palette_index: usize,
    pub wind_speed: f32,
    pub wind_angle: f32,
    pub fps: u32,
    pub seed: u64,
    pub effective_color_mode: ColorMode,
    pub no_sky: bool,
    pub storm_mode: bool,
    pub warp_strong: bool,
}
```

#### `ColorMode`

Color mode enumeration:

```rust
pub enum ColorMode {
    Auto,       // Auto-detect terminal capability
    TrueColor,  // 24-bit RGB
    Color256,   // 256-color palette
    Ansi16,     // 16 ANSI colors
}
```

#### Config Methods

- `effective_speed()`: Returns speed with storm-mode boost applied
- `effective_density()`: Returns density with storm-mode boost applied
- `warp_intensity()`: Returns domain-warp intensity (0.5 or 2.0)

### Simulation Module (`src/sim/`)

The simulation module handles the noise-field cloud simulation.

#### `Simulation`

Main simulation struct:

```rust
pub struct Simulation {
    pub noise: NoiseField,
    pub wind: Wind,
    pub time: f64,
}
```

#### Simulation Methods

- `new(seed: u64)`: Creates a new simulation with the given seed
- `update(dt: f64, config: &Config)`: Advances simulation by one timestep
- `sample_pixel(px: f64, py: f64, config: &Config) -> (f64, f64)`: Returns (opacity, shade) for a sub-pixel position
- `reseed(seed: u64)`: Reseeds noise fields and resets time

#### `NoiseField` (`src/sim/noise_field.rs`)

Handles Perlin noise sampling with fractal Brownian motion:

```rust
pub struct NoiseField {
    primary: Perlin,        // Main noise generator
    warp: Perlin,          // Domain warp noise
    octaves: Vec<(f64, f64)>, // (frequency, amplitude) per octave
    max_amplitude: f64,    // Sum of amplitudes for normalization
}
```

#### NoiseField Methods

- `new(seed: u32)`: Creates a new noise field with 4-octave fBm
- `sample_fbm(x: f64, y: f64, z: f64) -> f64`: Samples fractal Brownian motion at 3D coordinates
- `sample_warp(x: f64, y: f64, z: f64, intensity: f64) -> (f64, f64)`: Computes domain-warp displacement

#### `Wind` (`src/sim/wind.rs`)

Manages wind offset for cloud drift:

```rust
pub struct Wind {
    pub offset_x: f64,
    pub offset_y: f64,
}
```

#### Wind Methods

- `new()`: Creates a new wind with zero offset
- `update(speed: f32, angle_deg: f32, dt: f64)`: Advances wind offset by one timestep

### Render Module (`src/render/`)

The render module handles frame buffer management and ANSI emission.

#### `Renderer` (`src/render/mod.rs`)

Main renderer that manages double-buffered frame buffers:

```rust
pub struct Renderer {
    current: FrameBuffer,
    previous: FrameBuffer,
    palettes: Vec<Palette>,
    custom_palette: Option<Palette>,
    first_frame: bool,
}
```

#### Renderer Methods

- `new(width: u16, height: u16, custom_palette: Option<Palette>)`: Creates a new renderer
- `palette_count() -> usize`: Returns total number of available palettes
- `resize(width: u16, height: u16)`: Resizes both frame buffers
- `render_frame(sim: &Simulation, config: &Config, force_full: bool, buf: &mut Vec<u8>)`: Computes frame, diffs, and emits ANSI commands

#### `FrameBuffer` (`src/render/buffer.rs`)

Fixed-size grid of cells representing one frame:

```rust
pub struct FrameBuffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}
```

#### FrameBuffer Methods

- `new(width: u16, height: u16)`: Creates a new frame buffer
- `resize(width: u16, height: u16)`: Resizes and clears the buffer
- `get(x: u16, y: u16) -> &Cell`: Gets a cell at coordinates
- `set(x: u16, y: u16, cell: Cell)`: Sets a cell at coordinates

#### `Cell` (`src/render/halfblock.rs`)

Represents a terminal cell with two vertical sub-pixels:

```rust
pub struct Cell {
    pub top: Rgb,
    pub bottom: Rgb,
}
```

#### Cell Methods

- `new(top: Rgb, bottom: Rgb)`: Creates a new cell
- `same_as(&self, other: &Cell) -> bool`: Checks if cells match (for diffing)

#### Palette Functions (`src/render/palette.rs`)

- `builtin_palettes() -> Vec<Palette>`: Returns built-in color palettes
- `find_palette_index(name: &str) -> Option<usize>`: Finds palette by name
- `parse_custom_palette(s: &str) -> Option<Palette>`: Parses custom hex palette
- `blend(a: Rgb, b: Rgb, t: f64) -> Rgb`: Linear interpolation between colors
- `rgb_to_256(c: Rgb) -> u8`: Converts RGB to nearest 256-color index
- `rgb_to_ansi16(c: Rgb) -> u8`: Converts RGB to nearest ANSI-16 index
- `to_crossterm_color(rgb: Rgb, mode: ColorMode) -> Color`: Maps RGB to crossterm Color

### Terminal Module (`src/term/`)

The terminal module handles terminal state management and capability detection.

#### `TermGuard` (`src/term/guard.rs`)

RAII guard for terminal state:

```rust
pub struct TermGuard;
```

#### TermGuard Methods

- `new() -> Result<Self>`: Enables raw mode, enters alternate screen, hides cursor
- `teardown()`: Disables raw mode, leaves alternate screen, shows cursor, flushes

The `Drop` implementation automatically calls `teardown()` when the guard goes out of scope.

#### Capability Detection (`src/term/capability.rs`)

- `detect_color_mode() -> ColorMode`: Detects terminal color capability from environment variables

### Input Module (`src/input.rs`)

Handles non-blocking input polling.

#### Functions

- `poll_action() -> Option<Action>`: Non-blocking poll for keyboard input
- `key_to_action(key: KeyEvent) -> Option<Action>`: Maps key events to actions

### Actions Module (`src/actions.rs`)

Enumeration of runtime actions:

```rust
pub enum Action {
    IncreaseSpeed,
    DecreaseSpeed,
    DecreaseDensity,
    IncreaseDensity,
    CyclePalette,
    CycleWind,
    ToggleWarp,
    Reseed,
    ToggleStorm,
    Quit,
}
```

### Application Module (`src/app.rs`)

Main application loop and orchestration.

#### Functions

- `run(args: CliArgs) -> Result<()>`: Main application entry point
- `resolve_palette(name: &str) -> (usize, Option<Palette>)`: Resolves palette from name or hex
- `nearest_wind_angle_index(angle: f32) -> usize`: Finds nearest wind angle index

## Constants

### Simulation Constants (`src/sim/mod.rs`)

- `SPATIAL_SCALE: f64 = 0.03`: Spatial scale for noise sampling
- `TIME_SCALE: f64 = 0.15`: Time-axis scale for 3D noise
- `SHADE_OFFSET: f64 = 0.4`: Offset for shading sample

### Rendering Constants (`src/render/halfblock.rs`)

- `UPPER_HALF_BLOCK: char = '\u{2580}'`: The `▀` character used for half-block rendering

## Error Handling

The application uses `anyhow::Result` for error handling throughout. All functions that can fail return `Result<T>` where `T` is the return type.

## Dependencies

- `crossterm 0.28`: Terminal control (raw mode, alt screen, cursor, input)
- `clap 4`: CLI argument parsing
- `noise 0.9`: Perlin noise generation
- `anyhow 1`: Error handling
- `rand 0.8`: Random number generation for seeding
