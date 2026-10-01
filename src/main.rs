//! Sedimentary: a virtual pet rock that is deeply unimpressed with you.

use std::io::{self, Stdout, Write};
use std::panic;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::cursor::{Hide, Show};
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use sedimentary::app::App;
use sedimentary::clock::now_unix;
use sedimentary::dice::Dice;
use sedimentary::{save, ui};

/// How often the screen refreshes when nothing is pressed.
const FRAME: Duration = Duration::from_millis(250);

/// Put the terminal back the way we found it. Safe to call more than once.
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
}

/// Restores the terminal when dropped, including on early returns and errors.
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let guard = TerminalGuard;
        // Cleared here rather than with `Terminal::clear`, which asks the
        // terminal where its cursor is and fails if nobody answers.
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            Hide
        )?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    let started = Instant::now();
    loop {
        app.frame = (started.elapsed().as_millis() / FRAME.as_millis()) as u64 + 1;
        app.tick(now_unix());
        terminal.draw(|frame| ui::draw(frame, app))?;
        if app.should_quit {
            return Ok(());
        }
        if event::poll(FRAME)? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    app.handle_key(key, now_unix());
                }
                // The next draw resizes to fit.
                Event::Resize(..) => {}
                _ => {}
            }
        }
    }
}

const USAGE: &str = "\
Sedimentary: a virtual pet rock that is deeply unimpressed with you.

Usage: sedimentary [--save-path | --help | --version]

  --save-path   Print where your rock is saved, then exit.
  --help        Show this message.
  --version     Show the version.

Set SEDIMENTARY_SAVE to use a different save file.";

fn main() -> ExitCode {
    let save_path = save::default_save_path();
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "--save-path" => println!("{}", save_path.display()),
            "-h" | "--help" => println!("{USAGE}"),
            "-V" | "--version" => println!("sedimentary {}", env!("CARGO_PKG_VERSION")),
            other => {
                eprintln!("Unknown argument: {other}\n\n{USAGE}");
                return ExitCode::FAILURE;
            }
        }
        return ExitCode::SUCCESS;
    }

    // A panic must not leave the terminal in raw mode.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    let mut app = App::load(save_path, now_unix(), Dice::from_entropy());
    let result = (|| -> io::Result<()> {
        let _guard = TerminalGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        run(&mut terminal, &mut app)
    })();

    if let Err(e) = result {
        app.persist();
        eprintln!("Sedimentary hit a terminal error: {e}");
        eprintln!("Your rock was saved. It is unsurprised.");
        return ExitCode::FAILURE;
    }

    let mut out = io::stdout();
    if let Some(farewell) = app.farewell() {
        let _ = writeln!(out, "{farewell}");
    }
    match (&app.rock, app.saving_enabled) {
        (Some(_), true) => {
            let _ = writeln!(out, "Saved to {}", app.save_path.display());
        }
        (Some(_), false) => {
            let _ = writeln!(
                out,
                "Not saved this session (the old save file couldn't be moved aside)."
            );
        }
        (None, _) => {}
    }
    ExitCode::SUCCESS
}
