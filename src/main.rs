use crate::tui::modes::App;
use ::tui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, process};

use crate::tui::run_app;
use chip32_sim::{
    apf::parse_json,
    cpu::{HaltState, CPU},
};

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

    /// Override the default data slot to load and put into R0. Per the docs this defaults to the ID of data slot 0 from --data-json
    #[clap(short = 's', long, value_parser)]
    data_slot: Option<u32>,

    /// Run the program directly, without any GUI. Useful for testing
    #[clap(long)]
    disable_gui: bool,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let slots = args
        .data_json
        .map_or(None, |json_path| Some(parse_json(&json_path)));

    let mut state = CPU::load_file(&args.bin, slots, args.data_slot)?;

    if args.disable_gui {
        let mut log_length = 0;
        // No GUI, just run up to 1 million cycles
        for _ in 0..1_000_000 {
            state.step();

            if state.logs.len() > log_length {
                state
                    .logs
                    .iter()
                    .skip(log_length)
                    .for_each(|log| println!("{log}"));
                log_length = state.logs.len();
            }

            match state.halt {
                HaltState::Success => process::exit(0),
                HaltState::Failure => process::exit(1),
                _ => {}
            }
        }

        println!("Process did not terminate");

        process::exit(2);
    }

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
        println!("{:?}", err);
    }

    Ok(())
}
