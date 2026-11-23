use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal,
};
use quariumx::{
    cli::{parse_args, usage},
    fish::Aquarium,
    render::{install_panic_hook, Renderer, TerminalGuard},
};

fn main() {
    let config = match parse_args(std::env::args().skip(1)) {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("Error: {msg}\n\n{}", usage());
            std::process::exit(2);
        }
    };

    if config.show_help {
        println!("{}", usage());
        return;
    }

    if config.show_version {
        println!("quariumx {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    install_panic_hook();
    let guard = match TerminalGuard::enter() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Error: failed to initialize terminal: {err}");
            std::process::exit(1);
        }
    };

    let (width, height) = terminal::size().unwrap_or((80, 24));
    let mut rng = rand::thread_rng();
    let mut aquarium = Aquarium::new(width, height, config.fish_count, config.speed, &mut rng);
    let mut renderer = Renderer::new();

    let frame_duration = Duration::from_secs_f64(1.0 / f64::from(config.fps));
    let mut last_tick = Instant::now();
    let mut paused = false;
    let stdout = io::stdout();

    'main_loop: loop {
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32().min(0.1);
        last_tick = now;

        if !paused {
            aquarium.update(dt, &mut rng);
        }

        {
            let mut out = stdout.lock();
            if let Err(err) = renderer.render_to(&mut out, &aquarium, config.color) {
                drop(guard);
                eprintln!("Error: failed to render frame: {err}");
                std::process::exit(1);
            }
        }

        let next_frame = now + frame_duration;
        loop {
            let remaining = next_frame.saturating_duration_since(Instant::now());
            match event::poll(remaining) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                        match key.code {
                            KeyCode::Char('q' | 'Q') | KeyCode::Esc => break 'main_loop,
                            KeyCode::Char('c' | 'C' | 'd' | 'D') if ctrl => break 'main_loop,
                            KeyCode::Char(' ') => paused = !paused,
                            KeyCode::Char('+' | '=') => aquarium.add_fish(&mut rng),
                            KeyCode::Char('-') => aquarium.remove_fish(),
                            _ => {}
                        }
                    }
                    Ok(Event::Resize(w, h)) => {
                        aquarium.resize(w, h, &mut rng);
                    }
                    Ok(_) => {}
                    Err(_) => break 'main_loop,
                },
                Ok(false) => break,
                Err(_) => break 'main_loop,
            }
        }
    }
}
