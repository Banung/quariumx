# quariumx

A lightweight, flicker-free terminal ASCII aquarium simulation and screensaver written in Rust.

A vibrant marine ecosystem spawns directly inside your terminal and roams freely across its full width and height—turning organically more often near the edges, bouncing off walls, and layering multi-line creatures with transparent space compositing while preserving your terminal's native background transparency.

## Marine Ecosystem

- **Small fish** — 8 single-line ASCII species (`><>`, `><'>`, `>o>`, `><))>`, `>=:>`, `>))'>`, `>=>`, `><(('>`) darting across the water column
- **Medium fish** — 3-row directional reef species: tropical angelfish, spiny pufferfish, and striped clownfish
- **Large fish** — 5-row directional deep-water species: shark, swordfish, and whale cruising smoothly in the background layer
- **Jellyfish** — 4-row animated jellyfish drifting with pulsing bells and tentacles
- **Seabed crabs** — 2-row animated crabs with snapping claws and scuttling legs walking along the terminal floor
- **Seabed scenery** — swaying seaweed/kelp clusters and 4 multi-line ASCII coral reef formations (staghorn, brain coral, sea fan, and tube coral) layered behind all swimming creatures

## Build & Run

Run directly with Cargo:

```bash
cargo run
```

Build an optimized release binary:

```bash
cargo build --release
./target/release/quariumx
```

## Command-Line Options

```text
USAGE:
    quariumx [OPTIONS]
```

| Option | Description | Default |
| --- | --- | --- |
| `-n`, `--fish <N>` | Number of creatures (`1..=200`) | Auto-scaled at startup by terminal size |
| `-f`, `--fps <N>` | Target frames per second (`5..=120`) | `30` |
| `-s`, `--speed <F>` | Swimming speed multiplier (`0.1..=10.0`) | `1.0` |
| `--no-color` | Disable ANSI color output | Color enabled |
| `-h`, `--help` | Show help message and exit | |
| `-V`, `--version` | Show version information and exit | |

### Examples

```bash
# Run with 40 creatures at 60 FPS
quariumx --fish 40 --fps 60

# Run with slower swimming speed and monochrome output
quariumx -s 0.5 --no-color
```

## Interactive Controls

| Key | Action |
| --- | --- |
| `q` / `Q` / `Esc` / `Ctrl+C` / `Ctrl+D` | Quit and restore terminal |
| `Space` | Pause / resume simulation |
| `+` / `=` | Add a creature (up to 200) |
| `-` | Remove a creature (down to 1) |
