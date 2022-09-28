use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, sync::Mutex};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::cpu::State;

enum DisplayMode {
    Input,
    Memory { address: u16, state: TableState },
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    input: String,
    displayMode: DisplayMode,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            displayMode: DisplayMode::Input,
        }
    }
}

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
            }

            match app.displayMode {
                DisplayMode::Input => match key.code {
                    KeyCode::Enter => {
                        // TODO
                        // app.messages.push(app.input.drain(..).collect());
                        match app.input.as_str() {
                            "m" => {
                                app.displayMode = DisplayMode::Memory {
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
                    _ => {}
                },
                DisplayMode::Memory { .. } => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, state: &State) {
    match app.displayMode {
        DisplayMode::Input => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let msg = vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ];
            let style = Style::default();
            let mut text = Text::from(Spans::from(msg));
            text.patch_style(style);
            let help_message = Paragraph::new(text);
            f.render_widget(help_message, chunks[0]);

            let input = Paragraph::new(app.input.as_ref())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[1]);
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            );
        }
        DisplayMode::Memory {
            address,
            state: ref mut table_state,
        } => {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(5)
                .split(f.size());

            let rows = (0..10)
                .map(|i| {
                    let address = address + i * 4;

                    (address, state.ram.mem_read_long(address))
                })
                .map(|(address, data)| {
                    Row::new([
                        Cell::from(format!("{address:08X}")),
                        Cell::from(format!("{data:08X}")),
                    ])
                });

            let table = Table::new(rows)
                .header(Row::new([Cell::from("Address"), Cell::from("Data")]))
                .widths(&[Constraint::Length(10), Constraint::Min(100)]);

            f.render_stateful_widget(table, rects[0], table_state)
        }
    }
}
