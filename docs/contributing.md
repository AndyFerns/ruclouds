# Contributing

Contributions are welcome to ruclouds. This guide covers how to get started with development and the guidelines for contributing.

## Getting Started

### Prerequisites

- Rust 1.70 or later (2021 edition)
- Git
- A terminal for testing

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork:

```bash
git clone https://github.com/AndyFerns/ruclouds.git
cd ruclouds
```

3. Create a new branch for your feature or fix:

```bash
git checkout -b feature/your-feature-name
```

### Building

Build the project in debug mode:

```bash
cargo build
```

Build in release mode for performance testing:

```bash
cargo build --release
```

### Running

Run the application directly:

```bash
cargo run
```

Run with specific flags for testing:

```bash
cargo run -- --speed 2.0 --palette sunset
```

## Development Workflow

### Running Tests

Currently, ruclouds relies primarily on manual testing due to its visual nature. To test changes:

1. Build and run the application
2. Test the specific feature you're working on
3. Test edge cases (resize, different terminals, color modes)
4. Verify terminal cleanup on quit, Ctrl+C, and panic

### Manual Testing Checklist

When making changes, verify:

- Terminal is restored to normal state on quit (q key)
- Terminal is restored on Ctrl+C
- Terminal is restored on panic (temporarily add a panic!() to test)
- Resize works correctly (grow and shrink terminal window)
- All keybindings work as expected
- CLI flags parse correctly
- Color mode detection works on your terminal
- Animation is smooth at target FPS

### Platform Testing

Test on as many platforms as possible:

**Windows:**
- PowerShell
- pwsh (PowerShell 7)
- cmd.exe

**Unix:**
- bash with Kitty
- bash with Alacritty
- Other modern terminals (iTerm2, Hyper, WezTerm)

**macOS:**
- Terminal.app
- iTerm2

## Code Style

### Formatting

Use the standard Rust formatter:

```bash
cargo fmt
```

### Linting

Run clippy to catch common issues:

```bash
cargo clippy
```

### Documentation

Add doc comments to public APIs:

```rust
/// Sample fractal Brownian motion at `(x, y, z)`.
///
/// Returns a value in `[0.0, 1.0]`.
pub fn sample_fbm(&self, x: f64, y: f64, z: f64) -> f64 {
    // ...
}
```

## Project Structure

The codebase is organized into modules with clear responsibilities:

- `main.rs`: Entry point and panic hook
- `app.rs`: Main application loop
- `config.rs`: CLI parsing and configuration
- `actions.rs`: Action enumeration
- `input.rs`: Input polling
- `sim/`: Cloud simulation
- `render/`: Rendering and color management
- `term/`: Terminal state management

When adding new features, consider which module they belong in and maintain this separation of concerns.

## Key Architectural Principles

### Pure Function Sampling

The simulation's `sample_pixel` function must remain a pure function of `(px, py, time, wind_offset, config)`. It must never depend on grid dimensions. This is critical for resize correctness.

### Terminal Safety

Terminal state must always be restored on exit. The `TermGuard` RAII pattern and panic hook ensure this. Never bypass these mechanisms.

### Cross-Platform Compatibility

All code must work on both Windows and Unix. Use `crossterm` for terminal operations, not platform-specific APIs.

### Performance

Noise sampling is the hot path. Avoid allocations in the render loop. Precompute tables at startup when possible.

## Submitting Changes

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add custom palette parsing support
fix: correct wind direction calculation
docs: update installation instructions
```

### Pull Request Process

1. Push your branch to your fork
2. Create a pull request on GitHub
3. Describe your changes in the PR description
4. Link to any relevant issues
5. Wait for review

### PR Description Template

```markdown
## Description
Brief description of changes

## Testing
How you tested these changes

## Screenshots (if applicable)
Add screenshots for visual changes

## Checklist
- [ ] Code follows project style
- [ ] Tested on [platforms]
- [ ] Documentation updated
```

## Reporting Issues

When reporting bugs, include:

- Your operating system and version
- Terminal emulator and version
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected behavior
- Actual behavior
- Any relevant CLI flags used

## Feature Requests

For feature requests, describe:

- The feature you want
- Why it would be useful
- How you envision it working
- Any relevant examples or references

## Areas for Contribution

Some areas where contributions would be particularly welcome:

### Additional Palettes

Add more built-in color palettes to `src/render/palette.rs`. Look at existing palettes for the pattern.

### Noise Algorithms

Experiment with different noise algorithms or parameters in `src/sim/noise_field.rs` for different visual effects.

### Documentation

Improve documentation, add examples, or clarify complex sections.

### Performance

Profile and optimize performance, especially noise sampling or rendering.

### Cross-Platform Testing

Test on additional terminals and platforms, report issues, or fix compatibility problems.

### Build and Distribution

Improve the build process, add automated testing, or enhance release workflows.

## Inspiration

This project was inspired by lavat, a lava lamp simulation in the terminal. While lavat uses metaball-based simulation, ruclouds takes a different approach using noise-field cloud simulation to achieve wispy, organic cloud shapes.

Link to lavat: [to be added]

## Code of Conduct

Be respectful and constructive in all interactions. This is a hobby project and we want to keep the community welcoming.

## Questions

If you have questions about contributing or the codebase, feel free to:

- Open an issue with your question
- Start a discussion on GitHub
- Reach out through other channels if provided

## License

By contributing, you agree that your contributions will be licensed under the MIT License, same as the project.
