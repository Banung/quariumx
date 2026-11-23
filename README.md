# quariumx

A lightweight, flicker-free terminal ASCII aquarium simulation and screensaver written in Rust.

Small colorful ASCII fish spawn directly inside your terminal and roam freely across its full width and height—turning organically more often as they approach the edges and bouncing off the walls—while preserving your terminal's native background transparency.

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
| `-n`, `--fish <N>` | Number of fish (`1..=200`) | Auto-scaled at startup by terminal size (`4..=24`) |
| `-f`, `--fps <N>` | Target frames per second (`5..=120`) | `30` |
| `-s`, `--speed <F>` | Swimming speed multiplier (`0.1..=10.0`) | `1.0` |
| `--no-color` | Disable ANSI color output | Color enabled |
| `-h`, `--help` | Show help message and exit | |
| `-V`, `--version` | Show version information and exit | |

### Examples

```bash
# Run with 40 fish at 60 FPS
quariumx --fish 40 --fps 60

# Run with slower swimming speed and monochrome output
quariumx -s 0.5 --no-color
```

## Interactive Controls

| Key | Action |
| --- | --- |
| `q` / `Q` / `Esc` / `Ctrl+C` / `Ctrl+D` | Quit and restore terminal |
| `Space` | Pause / resume simulation |
| `+` / `=` | Add a fish (up to 200) |
| `-` | Remove a fish (down to 1) |
