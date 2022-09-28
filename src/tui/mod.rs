use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui::{backend::Backend, widgets::TableState, Frame, Terminal};

use crate::cpu::State;

use self::{
    main::render_main,
    memory::render_memory,
    modes::{App, DisplayMode},
};

mod main;
mod memory;
pub(crate) mod modes;

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut state: State,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app, &state))?;

        if let Event::Key(key) = event::read()? {
            if (key.code == KeyCode::Esc) {
                // Go to input
                app.displayMode = DisplayMode::Input;
                app.input = String::new();
            }

            match app.displayMode {
                DisplayMode::Input => match key.code {
                    KeyCode::Enter => {
                        match app.input.as_str() {
                            "m" => {
                                app.displayMode = DisplayMode::Memory {
                                    address: 0,
                                    state: TableState::default(),
                                };
                            }
                            "q" | "quit" => {
                                // Quit
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    _ => {}
                },
                DisplayMode::Memory { .. } => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, state: &State) {
    match app.displayMode {
        DisplayMode::Input => render_main(f, app, state),
        DisplayMode::Memory {
            address,
            state: ref mut table_state,
        } => render_memory(f, address, table_state, state),
    }
}
