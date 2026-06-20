# Configuration

ruclouds can be configured through CLI flags at launch and adjusted in real-time using keyboard shortcuts.

## CLI Flags

| Flag | Default | Description |
|---|---|---|
| `--speed <f32>` | `1.0` | Animation speed multiplier (0.1 to 5.0) |
| `--density <f32>` | `0.5` | Cloud density threshold (0.0 to 1.0) |
| `--palette <name\|hex,hex>` | `white-grey` | Palette name or custom hex pair |
| `--wind-speed <f32>` | `0.3` | Wind speed |
| `--wind-angle <f32>` | `0.0` | Wind direction in degrees (0 = left-to-right) |
| `--fps <u32>` | `30` | Target frames per second |
| `--seed <u64>` | random | RNG seed for reproducible results |
| `--color-mode <mode>` | `auto` | Color mode: auto, truecolor, 256, or ansi16 |
| `--no-sky` | off | Render against black instead of a sky gradient |

## Built-in Palettes

### white-grey
Classic white clouds on a blue sky. This is the default palette.

- Cloud light: RGB(245, 245, 250)
- Cloud dark: RGB(160, 165, 180)
- Sky top: RGB(135, 185, 235)
- Sky bottom: RGB(60, 110, 195)

### sunset
Warm orange/pink clouds on a sunset gradient.

- Cloud light: RGB(255, 200, 140)
- Cloud dark: RGB(200, 100, 80)
- Sky top: RGB(255, 155, 80)
- Sky bottom: RGB(120, 40, 100)

### midnight
Subdued grey-blue clouds on a deep dark sky.

- Cloud light: RGB(120, 130, 165)
- Cloud dark: RGB(50, 55, 80)
- Sky top: RGB(15, 20, 50)
- Sky bottom: RGB(5, 5, 20)

### storm
Heavy grey clouds on a dark overcast sky.

- Cloud light: RGB(145, 150, 160)
- Cloud dark: RGB(60, 62, 72)
- Sky top: RGB(80, 92, 115)
- Sky bottom: RGB(38, 42, 58)

## Custom Palettes

You can specify custom colors using hex color pairs:

```bash
ruclouds --palette "FF88CC,442255"
```

The format is two comma-separated hex colors (without the # prefix):
- First color: Light cloud color
- Second color: Dark cloud color

The sky gradient uses default blues when using custom palettes.

## Live Keybindings

All keys are polled non-blockingly every frame - they never pause the animation.

### Speed Control
- `+` or `=`: Increase animation speed (max 5.0)
- `-`: Decrease animation speed (min 0.1)

### Density Control
- `]`: Increase cloud density (max 1.0)
- `[`: Decrease cloud density (min 0.0)

### Palette Control
- `c`: Cycle to the next color palette

Cycles through built-in palettes in order: white-grey → sunset → midnight → storm → (back to white-grey)

### Wind Control
- `w`: Cycle wind direction

Cycles through 8 directions: 0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°

### Domain Warp
- `g`: Toggle domain-warp intensity

Toggles between subtle (0.5) and strong (2.0) domain warp intensity. Domain warp bends the coordinate space to create wispy cloud shapes.

### Simulation Control
- `r`: Reseed RNG and reset simulation time

Generates a new random seed and resets the simulation time to 0, creating a fresh cloud pattern. Wind offset is preserved for spatial continuity.

- `p`: Toggle storm mode

Storm mode boosts speed by 3x and increases density by 0.2 (capped at 1.0). Creates dramatic, fast-moving storm clouds.

### Quit
- `q` or `Esc`: Quit cleanly
- `Ctrl+C`: Quit cleanly (same path as q)

## Color Modes

### auto
Automatically detects terminal color capability by checking environment variables:
- `COLORTERM=truecolor` or `24bit`: truecolor
- `WT_SESSION` (Windows Terminal): truecolor
- `TERM_PROGRAM` (iTerm, Hyper, WezTerm, Alacritty): truecolor
- `TERM` containing `kitty` or `alacritty`: truecolor
- `TERM` containing `256color`: 256-color
- Default fallback: 256-color

### truecolor
Uses 24-bit RGB colors (16.7 million colors). Best for modern terminals that support it.

### 256
Uses xterm 256-color palette. Good fallback for terminals that don't support truecolor.

### ansi16
Uses standard 16 ANSI colors (VGA palette). Basic fallback for very old terminals.

## Configuration Examples

### Fast, Dense Storm

```bash
ruclouds --speed 2.0 --density 0.8 --palette storm --wind-speed 0.8
```

Then press `p` to enable storm mode for even more intensity.

> im aware of an issue which causes this to just render the base 4 terminal shades in 4 separate rectangles, and ill get to fixing that soon.

### Gentle Sunset
```bash
ruclouds --speed 0.5 --density 0.4 --palette sunset --wind-angle 315
```

Slow clouds drifting toward the upper-right.

### Custom Pink Theme
```bash
ruclouds --palette "FF66AA,992244" --no-sky
```

Pink clouds against your terminal's background.

### Reproducible Testing
```bash
ruclouds --seed 42 --speed 1.0 --density 0.5 --fps 30
```

Always produces the same cloud pattern for testing or comparison.
