use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, TableState},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

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

            match key.code {
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

                            app.input = String::new();
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
                KeyCode::Up => {
                    if let DisplayMode::Memory {
                        ref mut address,
                        state: _,
                    } = app.display_mode
                    {
                        if *address > 16 {
                            *address -= 16;
                        }
                    }
                }
                KeyCode::Down => {
                    if let DisplayMode::Memory {
                        ref mut address,
                        state: _,
                    } = app.display_mode
                    {
                        if *address < 8 * 1024 - (16 * 16) {
                            // Don't scroll past last page
                            *address += 16;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, state: &CPU) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    match app.display_mode {
        DisplayMode::Input(ref mut table_state) => {
            render_main(f, chunks.clone(), table_state, state)
        }
        DisplayMode::Memory {
            address,
            state: ref mut table_state,
        } => render_memory(f, chunks.clone(), address, table_state, state),
    }

    let input_paragraph = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input_paragraph, chunks[2]);
    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
    f.set_cursor(
        // Put cursor past the end of the input text
        chunks[2].x + app.input.width() as u16 + 1,
        // Move one line down, from the border to the input line
        chunks[2].y + 1,
    );
}
