use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui::{backend::Backend, widgets::TableState, Frame, Terminal};

use crate::cpu::CPU;

use self::{
    main::render_main,
    memory::render_memory,
    modes::{App, DisplayMode},
};

mod main;
mod memory;
pub(crate) mod modes;
pub(crate) mod util;

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut state: CPU,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app, &state))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Esc {
                // Go to input
                app.display_mode = DisplayMode::Input(TableState::default());
                app.input = String::new();
            }

            match app.display_mode {
                DisplayMode::Input(_) => match key.code {
                    KeyCode::Enter => {
                        match app.input.as_str() {
                            "s" => {
                                state.step();
                            }
                            "m" => {
                                app.display_mode = DisplayMode::Memory {
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, state: &CPU) {
    match app.display_mode {
        DisplayMode::Input(ref mut table_state) => {
            render_main(f, app.input.clone(), table_state, state)
        }
        DisplayMode::Memory {
            address,
            state: ref mut table_state,
        } => render_memory(f, address, table_state, state),
    }
}
