/// Runtime configuration parsed from command-line arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub fish_count: Option<usize>,
    pub fps: u16,
    pub speed: f32,
    pub color: bool,
    pub show_help: bool,
    pub show_version: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fish_count: None,
            fps: 30,
            speed: 1.0,
            color: true,
            show_help: false,
            show_version: false,
        }
    }
}

/// Parse command-line arguments (excluding `argv[0]`) into a [`Config`].
pub fn parse_args<I, S>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut config = Config::default();
    let mut iter = args.into_iter().map(Into::into);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                config.show_help = true;
            }
            "-V" | "--version" => {
                config.show_version = true;
            }
            "--no-color" => {
                config.color = false;
            }
            "-n" | "--fish" => {
                let val = iter
                    .next()
                    .ok_or_else(|| format!("Missing value for option '{arg}'"))?;
                config.fish_count = Some(parse_fish_count(&val)?);
            }
            "-f" | "--fps" => {
                let val = iter
                    .next()
                    .ok_or_else(|| format!("Missing value for option '{arg}'"))?;
                config.fps = parse_fps(&val)?;
            }
            "-s" | "--speed" => {
                let val = iter
                    .next()
                    .ok_or_else(|| format!("Missing value for option '{arg}'"))?;
                config.speed = parse_speed(&val)?;
            }
            _ if arg.starts_with("--fish=") => {
                let val = &arg["--fish=".len()..];
                config.fish_count = Some(parse_fish_count(val)?);
            }
            _ if arg.starts_with("--fps=") => {
                let val = &arg["--fps=".len()..];
                config.fps = parse_fps(val)?;
            }
            _ if arg.starts_with("--speed=") => {
                let val = &arg["--speed=".len()..];
                config.speed = parse_speed(val)?;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown option: '{arg}'"));
            }
            _ => {
                return Err(format!("Unexpected positional argument: '{arg}'"));
            }
        }
    }

    Ok(config)
}

fn parse_fish_count(raw: &str) -> Result<usize, String> {
    if raw.is_empty() {
        return Err("Missing value for --fish (-n)".to_string());
    }
    let count: usize = raw
        .parse()
        .map_err(|_| format!("Invalid fish count '{raw}': expected integer in 1..=200"))?;
    if !(1..=200).contains(&count) {
        return Err(format!(
            "Fish count {count} out of range: expected integer in 1..=200"
        ));
    }
    Ok(count)
}

fn parse_fps(raw: &str) -> Result<u16, String> {
    if raw.is_empty() {
        return Err("Missing value for --fps (-f)".to_string());
    }
    let fps: u16 = raw
        .parse()
        .map_err(|_| format!("Invalid FPS '{raw}': expected integer in 5..=120"))?;
    if !(5..=120).contains(&fps) {
        return Err(format!("FPS {fps} out of range: expected integer in 5..=120"));
    }
    Ok(fps)
}

fn parse_speed(raw: &str) -> Result<f32, String> {
    if raw.is_empty() {
        return Err("Missing value for --speed (-s)".to_string());
    }
    let speed: f32 = raw
        .parse()
        .map_err(|_| format!("Invalid speed '{raw}': expected number in 0.1..=10.0"))?;
    if !speed.is_finite() || !(0.1..=10.0).contains(&speed) {
        return Err(format!(
            "Speed '{raw}' out of range: expected finite number in 0.1..=10.0"
        ));
    }
    Ok(speed)
}

/// Return human-readable CLI usage and interactive keybindings help text.
pub fn usage() -> String {
    [
        "quariumx - terminal ASCII aquarium simulation",
        "",
        "USAGE:",
        "    quariumx [OPTIONS]",
        "",
        "OPTIONS:",
        "    -n, --fish <N>       Number of fish (1..=200, default: auto-scaled by terminal size)",
        "    -f, --fps <N>        Target frames per second (5..=120, default: 30)",
        "    -s, --speed <F>      Swimming speed multiplier (0.1..=10.0, default: 1.0)",
        "        --no-color       Disable ANSI color output",
        "    -h, --help           Show this help message and exit",
        "    -V, --version        Show version information and exit",
        "",
        "CONTROLS:",
        "    q / Esc / Ctrl+C / Ctrl+D   Quit aquarium",
        "    Space                       Pause / resume simulation",
        "    +                           Add a fish",
        "    -                           Remove a fish",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_spec() {
        let cfg = parse_args(Vec::<&str>::new()).expect("empty args should succeed");
        assert_eq!(
            cfg,
            Config {
                fish_count: None,
                fps: 30,
                speed: 1.0,
                color: true,
                show_help: false,
                show_version: false,
            }
        );
    }

    #[test]
    fn parses_short_flags() {
        let cfg = parse_args(["-n", "15", "-f", "60", "-s", "2.5", "-h", "-V"])
            .expect("valid short flags should parse");
        assert_eq!(cfg.fish_count, Some(15));
        assert_eq!(cfg.fps, 60);
        assert!((cfg.speed - 2.5).abs() < f32::EPSILON);
        assert!(cfg.color);
        assert!(cfg.show_help);
        assert!(cfg.show_version);
    }

    #[test]
    fn parses_long_flags_and_no_color() {
        let cfg = parse_args([
            "--fish",
            "200",
            "--fps",
            "120",
            "--speed",
            "0.1",
            "--no-color",
            "--help",
            "--version",
        ])
        .expect("valid long flags should parse");
        assert_eq!(cfg.fish_count, Some(200));
        assert_eq!(cfg.fps, 120);
        assert!((cfg.speed - 0.1).abs() < 1e-6);
        assert!(!cfg.color);
        assert!(cfg.show_help);
        assert!(cfg.show_version);
    }

    #[test]
    fn parses_equals_style_long_flags() {
        let cfg = parse_args(["--fish=1", "--fps=5", "--speed=10.0"])
            .expect("valid equals-style flags should parse");
        assert_eq!(cfg.fish_count, Some(1));
        assert_eq!(cfg.fps, 5);
        assert!((cfg.speed - 10.0).abs() < 1e-6);
    }

    #[test]
    fn rejects_invalid_or_out_of_range_fish_count() {
        assert!(parse_args(["-n", "0"]).is_err());
        assert!(parse_args(["--fish", "201"]).is_err());
        assert!(parse_args(["-n", "-5"]).is_err());
        assert!(parse_args(["-n", "abc"]).is_err());
        assert!(parse_args(["--fish="]).is_err());
    }

    #[test]
    fn rejects_invalid_or_out_of_range_fps() {
        assert!(parse_args(["-f", "4"]).is_err());
        assert!(parse_args(["--fps", "121"]).is_err());
        assert!(parse_args(["-f", "0"]).is_err());
        assert!(parse_args(["-f", "fast"]).is_err());
        assert!(parse_args(["--fps="]).is_err());
    }

    #[test]
    fn rejects_invalid_or_out_of_range_speed() {
        assert!(parse_args(["-s", "0.0"]).is_err());
        assert!(parse_args(["-s", "0.09"]).is_err());
        assert!(parse_args(["--speed", "10.1"]).is_err());
        assert!(parse_args(["-s", "-1.0"]).is_err());
        assert!(parse_args(["-s", "NaN"]).is_err());
        assert!(parse_args(["-s", "inf"]).is_err());
        assert!(parse_args(["-s", "-inf"]).is_err());
        assert!(parse_args(["-s", "slow"]).is_err());
        assert!(parse_args(["--speed="]).is_err());
    }

    #[test]
    fn rejects_missing_option_values() {
        assert!(parse_args(["-n"]).is_err());
        assert!(parse_args(["--fish"]).is_err());
        assert!(parse_args(["-f"]).is_err());
        assert!(parse_args(["--fps"]).is_err());
        assert!(parse_args(["-s"]).is_err());
        assert!(parse_args(["--speed"]).is_err());
    }

    #[test]
    fn rejects_unknown_flags_and_positional_args() {
        assert!(parse_args(["--unknown"]).is_err());
        assert!(parse_args(["-x"]).is_err());
        assert!(parse_args(["extra_positional"]).is_err());
        assert!(parse_args(["-n", "10", "unexpected"]).is_err());
    }

    #[test]
    fn usage_includes_flags_and_controls() {
        let text = usage();
        assert!(text.contains("--fish"));
        assert!(text.contains("--fps"));
        assert!(text.contains("--speed"));
        assert!(text.contains("--no-color"));
        assert!(text.contains("--help"));
        assert!(text.contains("--version"));
        assert!(text.contains("Space"));
        assert!(text.contains("Esc"));
        assert!(text.contains('+'));
        assert!(text.contains('-'));
    }
}
