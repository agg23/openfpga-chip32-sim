use crate::tui::modes::App;
use ::tui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use crate::tui::run_app;
use chip32_sim::cpu::CPU;

mod tui;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// The bin file to load
    #[clap(short, long, value_parser)]
    bin: String,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let state = CPU::load_file(&args.bin)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app, state);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
