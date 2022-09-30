use crate::tui::modes::App;
use ::tui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{fs, io};

use crate::tui::run_app;
use chip32_sim::{apf::DataJson, cpu::CPU};

mod tui;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// The bin file to load
    #[clap(short, long, value_parser)]
    bin: String,

    /// The data slot file to load
    #[clap(short, long, value_parser)]
    data_json: Option<String>,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let slots = args.data_json.map_or(None, |json_path| {
        let json = fs::read_to_string(json_path).expect("Could not find data slot JSON file");

        let data =
            serde_json::from_str::<DataJson>(&json).expect("Could not parse data slot JSON file");

        Some(data.data.data_slots)
    });

    let state = CPU::load_file(&args.bin, slots)?;

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
