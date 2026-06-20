# Changelog

## [0.5.0] - Current Release

### Overview
Version 0.5.0 represents the initial stable release of ruclouds, featuring a complete noise-field cloud simulation system with cross-platform terminal support, real-time rendering, and comprehensive customization options.

### Major Features

#### Core Simulation
- Implemented fractal Brownian motion (fBm) noise field with 4-octave sampling
- Added domain warping for wispy cloud shapes
- Implemented wind physics model for directional cloud drift
- Pure function sampling pipeline for seamless terminal resizing

#### Rendering System
- Half-block rendering technique for double vertical resolution
- FrameBuffer double buffering with diff-based ANSI emission
- Color palette system with 4 built-in schemes (white-grey, sunset, midnight, storm)
- Custom hex color palette support
- Multi-color mode support (truecolor, 256-color, 16-color with auto-detection)

#### Terminal Management
- RAII guard for terminal state safety (raw mode, alternate screen, cursor)
- Cross-platform color capability detection
- Panic hook for guaranteed terminal cleanup
- Real-time terminal resize handling without simulation reset

#### User Interface
- Non-blocking keyboard event polling
- Live keybindings for runtime parameter adjustment
- CLI argument parsing with clap
- Fixed-timestep main loop with configurable FPS

#### Development & Documentation
- Comprehensive ARCHITECTURE.md design document
- MkDocs documentation site with multiple sections
- GitHub Actions workflow for multi-platform binary releases
- MIT licensing

### Technical Implementation

#### Module Structure
- `config.rs` - CLI parsing and runtime configuration
- `actions.rs` - Action enumeration for keystrokes
- `input.rs` - Non-blocking input polling
- `sim/` - Cloud simulation (noise field, wind, sampling)
- `render/` - Rendering pipeline (buffer, half-block, palette)
- `term/` - Terminal state management (guard, capability detection)

#### Key Architectural Decisions
- Pure function sampling dependent only on (x, y, time, wind_offset, config)
- Double-buffer diffing to minimize terminal I/O
- RAII pattern for terminal state safety
- Cross-platform compatibility via crossterm

### Documentation
- Added comprehensive README with usage examples
- Created detailed documentation site with MkDocs
- Documented API reference and code structure
- Added contribution guidelines

### Bug Fixes & Improvements
- Fixed terminal cleanup on panic and Ctrl+C
- Improved color mode detection across platforms
- Optimized ANSI emission through frame diffing

### Known Issues
- Some terminals may have color palette selection issues (to be addressed in future)
- Storm mode may render base terminal shades on some configurations (to be fixed)

### Platform Support
- Windows: PowerShell, pwsh, cmd.exe
- Unix: bash, Kitty, Alacritty, and other modern terminals
- macOS: Terminal.app, iTerm2 (limited testing)

### Dependencies
- Rust 1.70+ (2021 edition)
- crossterm 0.28
- clap 4
- noise 0.9
- anyhow 1
- rand 0.8

---

## Development History

### Initial Development
The project was built following the ARCHITECTURE.md design document, implementing features in a logical order from terminal safety through simulation to rendering.

### Documentation Phase
Added comprehensive documentation including MkDocs setup, API reference, and contribution guidelines to support open-source development.

### Release Preparation
Configured GitHub Actions for multi-platform binary releases and updated repository metadata for distribution.
