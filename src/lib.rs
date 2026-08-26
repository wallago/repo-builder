//! **praline** - Build your repo like a lazy boss

/// Error handler implementation.
pub mod error;

/// Command-line arguments parser.
pub mod args;

/// Terminal user interface.
pub mod tui;

/// Config file.
pub mod config;

/// Main application.
pub mod app;

/// Helper functions.
pub mod help;

/// Common types that can be glob-imported for convenience.
pub mod prelude;

use std::io;

use args::Args;
use prelude::*;
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    config::Config,
    tui::{
        backend::Tui,
        command::{Command, input::InputCommand},
        event::{Event, EventHandler},
        state::{State, screen::Screen},
    },
};

/// Runs praline.
///
/// # Errors
///
/// Returns an error if the config fails to load or if the TUI fails to start.
pub fn run(args: &Args) -> Result<()> {
    let config = Config::load(args.config.as_deref())?;
    start_tui(args, config)
}

/// Starts the terminal user interface.
fn start_tui(args: &Args, config: Config) -> Result<()> {
    // Create an application.
    let mut state = State::new(args.accent_color, config)?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while state.running {
        // Expire any toast whose lifetime has elapsed before drawing.
        state.tick_toasts();
        // Render the user interface.
        tui.draw(&mut state)?;
        // Blink cursor
        state.cursor_active = state.last_blink.elapsed().as_millis() % 1000 < 500;
        // Handle events.
        match tui.events.next()? {
            Event::Key(key_event) => {
                let command = match state.screen_mode {
                    Screen::Form => Command::from_key(key_event, &state.keybindings),
                    Screen::Editing => Command::Input(InputCommand::parse(key_event, &state.input)),
                    Screen::Generated => Command::from_staged_key(key_event, &state.keybindings),
                    Screen::Exported => Command::from_exported_key(key_event, &state.keybindings),
                };
                state.run_command(command, tui.events.sender.clone())?;
            }
            Event::Mouse(_mouse_event) => {}
            Event::Tick | Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
